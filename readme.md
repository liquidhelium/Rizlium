# Rizlium
You're right, but this is an editor. (?)  

## TODO list:
 - [ ] ring colors
 - [x] fix strange ring y position
 - [x] fix strange ring x position
 - [ ] fix strange line width
   - 在横线后会出现.
 - [x] top and bottom mask 
 - [x] single notes
 - [ ] note textures
 - [ ] holds
 - [ ] hit particles
 - [ ] configurable log / warn / optimizations
 - [x] large game view cam
 - [x] dock window layout saving and loading
 - [x] recent files

 ## 已知问题
 1. ~~linux wayland 桌面录屏会导致音频状态异常，进而使谱面时间异常，原因未知~~
    解决方案: 使用`pipeware`可以解决.
 2. 使用bevy 0.12 和opengl后端时, 有时会出现奇怪的渲染问题 (见 bevyengine/bevy#10917 )
    解决方案: 使用vulkan做后端, 权宜之计.

 ## MSRV
   bevy 0.13 及以上需要 rustc 1.76 及以上, 请使用足够新的rust编译器.