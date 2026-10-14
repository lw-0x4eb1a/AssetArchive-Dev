fn check_manager(setup: &str, assertions: &str) {
    let lua = rlua::Lua::new();
    lua.context(|context| {
        context.load(include_str!("../scripts/class.lua")).exec().unwrap();
        context.load(r#"
            PLATFORM = "MACOS"
            install_count = 0
            valid_paths = { ["/custom/ffmpeg"] = true }
            local path_methods = {}
            path_methods.__index = path_methods
            function path_methods:as_string() return self.value end
            function path_methods:is_file() return valid_paths[self.value] == true end
            function path_methods.__div(parent, name)
                return setmetatable({ value = parent.value.."/"..name }, path_methods)
            end
            APP_BIN_DIR = setmetatable({ value = "/app/bin" }, path_methods)
            Persistant = { Config = {} }
            function Persistant.Config:Get(key) return self[key] end
            function Persistant.Config:SetAndSave(key, value) self[key] = value end
            IpcHandlers = {}
            function IpcHandlers.Register(name, handler) IpcHandlers[name] = handler end
            function print_error() end
            FFcore = {}
            function FFcore.InstallBundled()
                install_count = install_count + 1
                if install_error then error(install_error) end
                local path = "/app/bin/"..(PLATFORM == "WINDOWS" and "ffmpeg.exe" or "ffmpeg")
                valid_paths[path] = true
                return path
            end
            function FFcore.ValidateBinPath(path)
                if type(path) == "table" then path = path:as_string() end
                return valid_paths[path] == true, "invalid executable"
            end
        "#).exec().unwrap();
        context.load(setup).exec().unwrap();
        context.load(include_str!("../scripts/ffmpeg.lua")).exec().unwrap();
        context.load(assertions).exec().unwrap();
    });
}

#[test]
fn installs_at_startup_on_both_platforms() {
    for platform in ["MACOS", "WINDOWS"] {
        check_manager(&format!("PLATFORM = {:?}", platform), r#"
            assert(install_count == 1)
            assert(FFmpegManager:GetState().installed)
            assert(FFmpegManager:IsAvailable())
            local path, source = FFmpegManager:TryGetBinPath()
            assert(path == FFmpegManager.binpath:as_string())
            assert(source == "AUTO")
        "#);
    }
}

#[test]
fn prefers_custom_path_and_falls_back_when_invalid() {
    check_manager("Persistant.Config.ffmpeg_path = '/custom/ffmpeg'", r#"
        assert(install_count == 1)
        local path, source = FFmpegManager:TryGetBinPath()
        assert(path == "/custom/ffmpeg" and source == "CUSTOM")
        assert(Persistant.Config.ffmpeg_path == "/custom/ffmpeg")
        valid_paths["/custom/ffmpeg"] = false
        path, source = FFmpegManager:TryGetBinPath()
        assert(path == "/app/bin/ffmpeg" and source == "AUTO")
    "#);
}

#[test]
fn validates_custom_settings_and_restores_default() {
    check_manager("", r#"
        assert(IpcHandlers.ffmpeg_custom_install({ path = "/custom/ffmpeg" }).success)
        assert(not IpcHandlers.ffmpeg_custom_install({ path = "/missing/ffmpeg" }).success)
        assert(Persistant.Config.ffmpeg_path == "/custom/ffmpeg")
        assert(FFmpegManager:GetState().custom_installed)
        assert(IpcHandlers.ffmpeg_custom_install({ path = "" }).success)
        assert(Persistant.Config.ffmpeg_path == "")
        assert(not FFmpegManager:GetState().custom_installed)
        assert(FFmpegManager:TryGetBinPath() == "/app/bin/ffmpeg")
    "#);
}

#[test]
fn keeps_custom_path_available_when_unpack_fails() {
    check_manager(r#"
        install_error = "permission denied"
        Persistant.Config.ffmpeg_path = "/custom/ffmpeg"
    "#, r#"
        assert(FFmpegManager:TryGetBinPath() == "/custom/ffmpeg")
        local state = FFmpegManager:GetState()
        assert(not state.installed and state.custom_installed)
        assert(state.install_error:find("permission denied"))
    "#);
}

#[test]
fn reports_unpack_failure_and_allows_retry() {
    check_manager("install_error = 'permission denied'", r#"
        assert(not FFmpegManager:IsAvailable())
        assert(not pcall(function() FFmpegManager:TryGetBinPath() end))
        assert(not IpcHandlers.ffmpeg_install().success)
        install_error = nil
        assert(IpcHandlers.ffmpeg_install().success)
        assert(FFmpegManager:IsAvailable())
        assert(FFmpegManager:GetState().install_error == nil)
    "#);
}
