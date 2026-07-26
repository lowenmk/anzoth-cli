@echo off
setlocal
set "SCRIPT_DIR=%~dp0"
set "EXECUTABLE=%SCRIPT_DIR%codex-rs\target\debug\anzoth.exe"
set "MODEL_CATALOG=%SCRIPT_DIR%codex-rs\models-anzoth.json"

if defined ANZOTH_COMPAT_MODE (
    if not defined ANZOTH_HOME (
        if defined CODEX_HOME (
            set "ANZOTH_HOME=%CODEX_HOME%"
        ) else (
            set "ANZOTH_HOME=%USERPROFILE%\.codex"
        )
    )
    if not defined ANZOTH_API_KEY if defined OPENAI_API_KEY set "ANZOTH_API_KEY=%OPENAI_API_KEY%"
    if not defined ANZOTH_API_KEY if defined CODEX_API_KEY set "ANZOTH_API_KEY=%CODEX_API_KEY%"
) else (
    if not defined ANZOTH_HOME set "ANZOTH_HOME=%USERPROFILE%\.anzoth"
    set "CODEX_HOME=%ANZOTH_HOME%"
    if not defined ANZOTH_API_KEY if defined OPENAI_API_KEY set "ANZOTH_API_KEY=%OPENAI_API_KEY%"
    if not defined ANZOTH_API_KEY if defined CODEX_API_KEY set "ANZOTH_API_KEY=%CODEX_API_KEY%"
)

if not defined ANZOTH_COMPAT_MODE (
    if not exist "%ANZOTH_HOME%" mkdir "%ANZOTH_HOME%"
    if not exist "%ANZOTH_HOME%\config.toml" (
        >"%ANZOTH_HOME%\config.toml" (
            echo model_provider = 'anzoth'
            echo model = 'Anzoth-Core'
            echo model_catalog_json = '%MODEL_CATALOG%'
            echo forced_login_method = 'api'
        )
    )
)

if not exist "%EXECUTABLE%" (
    echo Missing executable: "%EXECUTABLE%"
    exit /b 1
)

pushd "%SCRIPT_DIR%"
"%EXECUTABLE%" %*
set "EXIT_CODE=%ERRORLEVEL%"
popd
exit /b %EXIT_CODE%
