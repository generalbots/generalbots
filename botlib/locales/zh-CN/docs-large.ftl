# Docs 大型 .docx 加载路径 (#2533) — zh-CN 目录。8 个区域设置使用相同的键集合。

docs-large-note-local = 仅本地 — 预览，不是服务器加载路径
docs-large-deferred = 延期: 流式导入、服务器请求体限制、gzip/brotli 传输、服务器端 ETag/If-Match 校验、异步导出以及实测基线。
docs-large-open = 大型文档加载路径 (本地预览)
docs-large-title = 大型文档加载路径
docs-large-close = 关闭

docs-large-size-label = 模拟大小
docs-large-size-5 = 5 MB
docs-large-size-10 = 10 MB
docs-large-size-20 = 20 MB
docs-large-load = 加载测试数据
docs-large-controls = 测试数据与保存控制
docs-large-mode-label = 保存模式
docs-large-mode-auto = 自动
docs-large-mode-manual = 手动
docs-large-toggle-mode = 切换自动/手动
docs-large-etag-label = ETag (If-Match)
docs-large-save-incremental = 保存已更改的块
docs-large-save-full = 保存完整正文
docs-large-bytes-label = 传输字节数 (估算)
docs-large-bytes-incremental = 增量字节数: { $n }

docs-large-degradation = 编辑状态
docs-large-degraded-banner = 编辑会很慢 — 已以只读预览方式打开
docs-large-edit-ready = 编辑已启用 — 文档在阈值以内
docs-large-enable-anyway = 仍然启用编辑

docs-large-window-title = 窗口化编辑
docs-large-range-label = 已挂载范围
docs-large-scroll-label = 滚动位置
docs-large-window = 已挂载范围: { $s }–{ $e } / { $t }
docs-large-scope = contenteditable 仅限于可见窗口
docs-large-inspector = 检查器
docs-large-queue-depth = 队列深度: { $n }

docs-large-baseline-title = 基线 (本地估算)
docs-large-baseline-local = 本地估算 — 不是浏览器/服务器的实测数值
docs-large-col-size = 大小
docs-large-col-heap = 堆峰值估算
docs-large-col-ttfp = 首次绘制时间
docs-large-col-keystroke = 按键延迟
docs-large-col-autosave = 自动保存时长
docs-large-col-wire = 传输字节

docs-large-save-ok = 已保存 { $n } 字节
docs-large-save-stale = 已拒绝过期保存 — ETag 不匹配，数据未丢失
docs-large-save-413 = 文档过大，无法完整保存 — 请使用增量 / 拆分
docs-large-save-inc-413 = 更改批次过大 — 请拆分；数据未丢失

docs-large-status-loaded = 测试数据已加载: { $n } 个块
docs-large-status-mode = 保存模式: { $s }
docs-large-status-enabled = 编辑已启用
