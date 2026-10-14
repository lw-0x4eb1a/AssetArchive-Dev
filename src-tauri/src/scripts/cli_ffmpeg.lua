require "ffmpeg"

if Args.path == nil then
	local path, source = FFmpegManager:TryGetBinPath("no_warning")
	if path == nil then
		print_error("[ERROR] FFmpeg is unavailable: "..(FFmpegManager.install_error or "no valid executable"))
		exit(1)
	end
	if source == "CUSTOM" then
		print_info("[INFO] Using custom FFmpeg: "..path)
	else
		print_info("[INFO] Using bundled FFmpeg: "..path)
	end
	exit(0)
elseif Args.path == "" then
	Persistant.Config:SetAndSave("ffmpeg_path", "")
	print_info("[INFO] Cleared custom path; using bundled FFmpeg")
	exit(0)
else
	print_info("[INFO] Validating path: "..Args.path)
	local success, message = FFcore.ValidateBinPath(Args.path)
	if success then
		Persistant.Config:SetAndSave("ffmpeg_path", Args.path)
		print_info("[INFO] Successfully setup custom FFmpeg path")
	else
		print_error("[ERROR] Failed to setup path: "..message)
	end
	exit(success and 0 or 1)
end
