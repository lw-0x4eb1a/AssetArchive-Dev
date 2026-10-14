import React, { useCallback, useEffect, useReducer, useState } from 'react'
import style from './index.module.css'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { Button, H5, InputGroup, Tag } from '@blueprintjs/core'
import { useLuaCall, useLuaCallOnce } from '../../hooks'
import { emit } from '@tauri-apps/api/event'

export default function FFmpegInstaller() {
  const [ffmpegState, setFState] = useState({
    checking: true, installed: false, custom_installed: false, custom_path: "", install_error: ""
  })
  const [path, setPath] = useState("")
  const [flag, forceUpdate] = useReducer(value=> value + 1, 0)
  useLuaCallOnce<string>("ffmpeg_getstate", response=> {
    const state = JSON.parse(response)
    setFState({checking: false, ...state})
    setPath(state.custom_path || "")
  }, {}, [flag])

  useEffect(()=> {
    emit("ffmpeg_installed")
  }, [flag])

  const configure = useLuaCall<string>("ffmpeg_custom_install", response=> {
    const data = JSON.parse(response)
    if (data.success) {
      forceUpdate()
    }
    else {
      window.alert("设置失败：\n" + data.message)
    }
  }, {}, [])

  const restore = useLuaCall<string>("ffmpeg_install", response=> {
    const data = JSON.parse(response)
    if (!data.success) {
      window.alert("恢复失败：\n" + data.message)
    }
    forceUpdate()
  }, {}, [])

  const confirm = useCallback(()=> {
    const value = path.trim()
    if (value) {
      configure({path: value})
    }
    else {
      window.alert("请输入有效的路径，或点击恢复默认。")
    }
  }, [path, configure])

  const onKeyDown = useCallback((event: React.KeyboardEvent)=> {
    if (event.key === "Enter") {
      confirm()
    }
  }, [confirm])

  return (
    <div className={style["main"]}>
      <div className={style["box"]}>
        <H5>FFmpeg视频编码器</H5>
        <Tag intent={ffmpegState.checking ? "none" : ffmpegState.custom_installed || ffmpegState.installed ? "success" : "warning"}>
          {ffmpegState.checking ? "检查中..." : ffmpegState.custom_installed ? "使用自定义路径" : ffmpegState.installed ? "使用内置版本" : "不可用"}
        </Tag>
        <div className={style["group"]}>
          <p>默认使用内置版本，无需配置。</p>
          {!ffmpegState.checking && !ffmpegState.installed && <>
            <p>{ffmpegState.install_error || "内置版本不可用，请尝试恢复。"}</p>
            <Button icon="refresh" onClick={()=> restore()}>恢复内置版本</Button>
          </>}
        </div>
        <div className={style["group"]}>
          <p>自定义可执行路径：</p>
          <InputGroup
            fill
            autoComplete="off"
            autoCorrect="none"
            spellCheck={false}
            placeholder="输入FFmpeg路径"
            value={path}
            onChange={event=> setPath(event.currentTarget.value)}
            onKeyDown={onKeyDown}
            rightElement={<Button icon="small-tick" disabled={ffmpegState.checking} onClick={confirm}/>}
          />
          <div className={style["hint"] + " bp4-running-text"}>
            <p>建议使用绝对路径，例如：</p>
            <p className={style["path"]}>C:/tools/ffmpeg.exe</p>
            <p className={style["path"]}>/opt/homebrew/bin/ffmpeg</p>
            <p>自定义版本需支持libx264、gif、png编码，以及mp4、gif、mov封装。</p>
            {ffmpegState.custom_path && !ffmpegState.custom_installed &&
              <p>自定义路径无效，将尝试使用内置版本。</p>}
          </div>
          <Button icon="reset" disabled={ffmpegState.checking || !ffmpegState.custom_path}
            onClick={()=> configure({path: ""})}>恢复默认</Button>
        </div>
      </div>
    </div>
  )
}

export async function openInstaller() {
  let label = "ffmpeg-installer"
  let subwindow = await WebviewWindow.getByLabel(label)
  if (subwindow)
    subwindow.setFocus().then(console.log, console.error)
  else
    new WebviewWindow(label, {
      title: "",
      url: "/ffmpeg-installer",
      width: 340,
      height: 480,
      resizable: false,
    })
}
