# Alerter 使用指引（macOS 通知交互）

`alerter` 是一个命令行工具，用于在 macOS 上发送原生系统通知，可带输入框或操作按钮，支持 macOS 10.8+（包括 Catalina）。

可用于：
- 🚀 任务开始时通知
- ✅ 任务完成时通知
- 🧭 中间步骤需要用户确认
- ✏️ 请求用户输入

---

## 1) 普通通知（任务开始/完成）
```bash
# 任务开始
alerter -title "任务开始" -message "已开始执行 🚀" -sound default -group JOB1

# 任务完成
alerter -title "任务完成" -message "✅ 执行成功" -timeout 8 -group JOB1
```

## 2) 确认型通知（Actions）
```bash
ANSWER=$(alerter -message "是否继续执行？" -actions "继续","取消" -timeout 15)
case "$ANSWER" in
  "继续") echo "继续执行" ;;
  "取消") echo "已取消" ;;
  "@TIMEOUT") echo "超时未响应" ;;
esac
```

## 2) 确认型通知（Actions）
```bash
INPUT=$(alerter -reply -title "参数输入" -message "请输入版本号：")
echo "用户输入：$INPUT"
```

## example

### 📊 任务状态通知（status）
```bash
# requirements 阶段完成
alerter -title "📋 Requirements" -message "需求分析已生成 ✅" > /dev/null 2>&1 &

# design 阶段完成
alerter -title "📐 Design" -message "设计文档已生成 ✅" > /dev/null 2>&1 &

# tasks 阶段完成
alerter -title "🛠 Tasks" -message "任务列表已生成 ✅" > /dev/null 2>&1 &

# tasks phase 完成
alerter -title "📦 Tasks Phase" -message "任务执行阶段完成 ✅" > /dev/null 2>&1 &

# 所有任务执行完成
alerter -title "🏁 Tasks Done" -message "所有任务已完成 ✅" > /dev/null 2>&1 &
```

###  整体执行完成（complete）

```bash
alerter -title "✨ 执行完成" -message "整个 prompt 已处理完成 🎉" > /dev/null 2>&1 &
```

### 用户确认或输入（confirm）
- 确认型通知

```bash
 ANSWER=$(alerter -message "是否开始部署？" -actions "开始","取消" -timeout 15)
if [ "$ANSWER" = "开始" ]; then
echo "用户确认执行"
fi
```

- 回复型通知（获取用户输入）
```bash
INPUT=$(alerter -reply -title "参数输入" -message "请输入版本号：")
echo "用户输入：$INPUT" 
```