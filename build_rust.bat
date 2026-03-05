@echo off  
echo ========================================  
echo   NoteCypher Desktop App - Build Script  
echo ========================================  
echo.  
echo Checking Rust installation...  
rustc --version  
if errorlevel 1 (  
    echo ERROR: Rust is not installed or not in PATH  
    echo Please install Rust from https://rustup.rs/  
    pause  
    exit /b 1  
)  
echo.  
echo Rust is installed!  
echo.  
echo Building NoteCypher in Release mode...  
echo This may take a while on first build...  
echo.  
cargo build --release  
  
if errorlevel 1 (  
    echo.  
    echo ERROR: Build failed!  
    pause  
    exit /b 1  
)  
  
echo ========================================  
echo   Build Successful!  
echo ========================================  
echo.  
echo The executable is located at:  
echo   target\release\notecypher.exe  
echo.  
pause 
