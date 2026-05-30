1. 构建本应用，然后安装。
2. 本应用中看到本地的端口，替换到下面的内容。默认是 53789 端口。
3. 
## 参考 Hooks 配置
```json
{

  "hooks": {
    "PostToolBatch": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "curl -sS -X POST 'http://127.0.0.1:53786/signal?color=green&delay=100'",
            "timeout": 2
          }
        ]
      }
    ],
    "SessionStart": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "curl -sS -X POST 'http://127.0.0.1:53786/signal?color=red&delay=0'",
            "timeout": 2
          }
        ]
      }
    ],
     "SessionEnd": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "curl -sS -X POST 'http://127.0.0.1:53786/signal?color=red&delay=0'",
            "timeout": 2
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "curl -sS -X POST 'http://127.0.0.1:53786/signal?color=green&delay=100'",
            "timeout": 2
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "curl -sS -X POST 'http://127.0.0.1:53786/signal?color=green&delay=100'",
            "timeout": 2
          }
        ]
      }
    ],
    "PermissionRequest": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "curl -sS -X POST 'http://127.0.0.1:53786/signal?color=yellow&delay=100'",
            "timeout": 2
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "curl -sS -X POST 'http://127.0.0.1:53786/signal?color=red&delay=0'",
            "timeout": 2
          }
        ]
      }
    ]
  }
}

```

