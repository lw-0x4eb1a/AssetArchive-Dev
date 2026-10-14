local Config = Persistant.Config

local FFmpeg = Class(function(self)
    self.bindir = APP_BIN_DIR
    self.binname = PLATFORM == "WINDOWS" and "ffmpeg.exe" or "ffmpeg"
    self.binpath = self.bindir/self.binname
    self:Install()
end)

function FFmpeg:GetState()
    local installed = self.binpath:is_file()
        and self:ValidateBinPath(self.binpath)
    local custom_path = Config:Get("ffmpeg_path")
    local custom_installed = custom_path ~= nil and custom_path ~= ""
        and self:ValidateBinPath(custom_path)

    return {
        installed = installed,
        custom_installed = custom_installed,
        custom_path = custom_path,
        install_error = self.install_error,
    }
end

function FFmpeg:IsAvailable()
    return self:TryGetBinPath("no_warning") ~= nil
end

function FFmpeg:Install()
    local success, message = pcall(FFcore.InstallBundled)
    if success then
        success, message = self:ValidateBinPath(self.binpath)
    end
    self.install_error = not success and tostring(message) or nil
    if not success then
        print_error("[ERROR] Failed to install bundled FFmpeg: "..self.install_error)
    end
    return success, self.install_error
end

function FFmpeg:ValidateBinPath(path)
    return FFcore.ValidateBinPath(path)
end

function FFmpeg:TryGetBinPath(no_warning)
    local path = Config:Get("ffmpeg_path")
    if path ~= nil and path ~= "" and self:ValidateBinPath(path) then
        return path, "CUSTOM"
    elseif self:ValidateBinPath(self.binpath:as_string()) then
        return self.binpath:as_string(), "AUTO"
    elseif not no_warning then
        error("Failed to get ffmpeg path: "..(self.install_error or "no valid executable"))
    end
end

FFmpegManager = FFmpeg()

IpcHandlers.Register("ffmpeg_install", function()
    local success, message = FFmpegManager:Install()
    return { success = success, message = message }
end)

IpcHandlers.Register("ffmpeg_getstate", function()
    return FFmpegManager:GetState()
end)

IpcHandlers.Register("ffmpeg_custom_install", function(param)
    assert(type(param) == "table")
    assert(type(param.path) == "string")
    if param.path == "" then
        Config:SetAndSave("ffmpeg_path", "")
        return { success = true }
    else
        local success, message = FFcore.ValidateBinPath(param.path)
        if success then
            Config:SetAndSave("ffmpeg_path", param.path)
            return {
                success = true,
            }
        else
            return {
                success = false,
                message = message,
            }
        end
    end
end)
