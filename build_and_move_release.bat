START /B /wait cargo build --release -F integration_tests

echo off

timeout /t 1

echo on

copy /Y /B "%~dp0target\release\gdext_coroutines.dll" "%~dp0tester\Bin\gdext_coroutines.dll"

pause