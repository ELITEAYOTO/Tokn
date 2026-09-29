@echo off
setlocal EnableExtensions

set "VSWHERE=C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "%VSWHERE%" set "VSWHERE=%ProgramFiles%\Microsoft Visual Studio\Installer\vswhere.exe"

if not exist "%VSWHERE%" (
  echo [Tokn] vswhere.exe introuvable.
  exit /b 2
)

for /f "usebackq tokens=*" %%I in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VSROOT=%%I"

if not defined VSROOT (
  echo [Tokn] Aucun toolset MSVC x64/x86 complet detecte.
  exit /b 3
)

set "DEVCMD=%VSROOT%\Common7\Tools\VsDevCmd.bat"
if not exist "%DEVCMD%" (
  echo [Tokn] VsDevCmd.bat introuvable.
  exit /b 4
)

call "%DEVCMD%" -arch=x64 -host_arch=x64 >nul
if errorlevel 1 (
  echo [Tokn] Initialisation MSVC echouee.
  exit /b 5
)

%*
exit /b %ERRORLEVEL%
