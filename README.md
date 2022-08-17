# 基于Bevy引擎的多人对战俄罗斯方块

开发环境/工具：Rust、Bevy、Tokio、Tonic、ProtocolBuffer

使用Rust和Bevy游戏引擎开发的多人对战型俄罗斯方块游戏，使用Tokio异步框架和Tonic构建服务端程序用于统计各客户端数据排名得分信息，使用protobuf作为信息传输媒介，使用Bevy游戏引擎制作俄罗斯方块客户端。具有暂停、重启、方块变换、方块预览、得分统计、实时排名等功能。
