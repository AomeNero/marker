# 帮助页工具网格紧凑布局实现计划

> **致代理执行者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 按任务执行本计划。步骤使用复选框（`- [ ]`）语法跟踪。

**目标：** 将帮助页「标注会话工具」区做成完整的 2×5 网格 + 紧凑行内卡片，使序号不再在右侧留下空位。

**架构：** 仅 CSS 修改 `docs/styles.css` 中既有的 `.help-tool-grid` / `.help-tool-card` 规则。在每张卡片上使用 CSS Grid，使顶层 `kbd` 与 `h3`+`p` 并排而无需改 HTML。移动端单列 media query 保持不变。

**技术栈：** 静态 HTML 帮助页（`docs/help.html`）、`docs/styles.css`

## 全局约束

- 范围仅限帮助文档样式——不要改动商店截图、i18n 字符串或应用 Vue UI
- 优先纯 CSS；仅当 CSS 无法对齐布局时才动 `docs/help.html`
- 保留滚动显现（`t-scroll-reveal`、`data-reveal-delay`）与悬停样式
- 序号 `<p>` 内嵌套的 `<kbd>` 须保持行内按键样式；仅直接子级快捷键 `kbd` 移入侧列
- `max-width: 1024px` 时 `.help-tool-grid` 保持 `grid-template-columns: 1fr`

**规格：** `docs/superpowers/specs/2026-07-22-help-tools-grid-compact-design.md`

---

## 文件映射

| 文件 | 角色 |
|------|------|
| `docs/styles.css` | 工具网格改双列；卡片内部紧凑布局 |
| `docs/help.html` | 预期无改动（仅验证） |

---

### 任务 1：双列紧凑工具卡片

**文件：**
- 修改：`docs/styles.css`（`.help-tool-grid` 约 1358–1361，`.help-tool-card` 约 1363–1400）
- 验证：`docs/help.html`（工具区约 257–313）——标记不变

**接口：**
- 消费：既有 DOM——每个 `.help-tool-card` 为 `kbd` + `h3` + `p`（序号 `p` 可能含嵌套 `kbd`）
- 产出：桌面 2×5 满网格；紧凑水平卡片布局

- [ ] **步骤 1：将 `.help-tool-grid` 改为双列**

在 `docs/styles.css` 中，将：

```css
.help-tool-grid {
  grid-template-columns: repeat(3, minmax(0, 1fr));
  margin-bottom: 16px;
}
```

替换为：

```css
.help-tool-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin-bottom: 16px;
}
```

- [ ] **步骤 2：将 `.help-tool-card` 重排为紧凑双列网格**

将从 `.help-tool-card` 到 `.help-tool-card p` 的卡片块替换为：

```css
.help-tool-card {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  grid-template-rows: auto auto;
  column-gap: 12px;
  row-gap: 2px;
  align-items: start;
  padding: 12px 14px;
  border: 1px solid var(--line);
  border-radius: 10px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(250, 250, 245, 0.98)),
    var(--surface);
  box-shadow: 0 1px 0 rgba(18, 18, 18, 0.04);
  transition:
    box-shadow var(--duration-fast) var(--ease-smooth-out),
    transform var(--duration-fast) var(--ease-smooth-out);
}

.help-tool-card:hover {
  box-shadow: 0 12px 28px rgba(18, 18, 18, 0.08);
  transform: translateY(-2px);
}

.help-tool-card.t-scroll-reveal.is-revealed:hover {
  transform: translateY(-2px);
}

.help-tool-card > kbd {
  grid-column: 1;
  grid-row: 1 / span 2;
  align-self: start;
  margin-bottom: 0;
}

.help-tool-card h3 {
  grid-column: 2;
  margin: 0;
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: 950;
  line-height: 1.2;
}

.help-tool-card p {
  grid-column: 2;
  margin: 0;
  color: var(--muted);
  font-size: 13px;
  line-height: 1.5;
}
```

**不要**改动将 `.help-tool-grid` 设为 `grid-template-columns: 1fr` 的 `@media (max-width: 1024px)` 规则。

- [ ] **步骤 3：浏览器视觉验证**

打开 `docs/help.html`（file URL 或任意本地静态服务）。检查：

1. 桌面（~1200px+）：10 张卡片 2 列 × 5 行——序号旁无空位
2. 序号描述完整可读；段落内嵌套 `kbd` 仍呈按键样式
3. 悬停抬升仍有效；滚动显现仍运行
4. 窄视口（≤1024px）：单列堆叠

预期：四项检查全部通过。

- [ ] **步骤 4：提交**

```bash
git add docs/styles.css
git commit -m "ui(docs): compact help tools grid to two columns"
```

---

## 规格覆盖（自审）

| 规格要求 | 任务 |
|------------------|------|
| 桌面 2×5 网格、无孤立卡片 | 任务 1 步骤 1 |
| 行内紧凑 kbd + 文本 | 任务 1 步骤 2 |
| 移动单列保留 | 任务 1 步骤 2 注 + 步骤 3 |
| 无文案 / i18n / 截图改动 | 全局约束 |
| 悬停 / 显现保留 | 任务 1 步骤 2 + 步骤 3 |
| 序号长描述可读 | 任务 1 步骤 3 |

无占位符。单一子系统——一份计划即可。
