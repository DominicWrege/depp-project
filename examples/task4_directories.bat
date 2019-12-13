@echo off

set docname=test.tex

mkdir "Schriftstuecke"
cd ".\Schriftstuecke\"
mkdir "Alt"
mkdir "Neu"
cd ".\Alt\"
mkdir "Latex"
cd "..\Neu\"
```
mkdir "Latex"
cd ".."
FOR /D /R %%A IN (Schriftstuecke) DO (
    rem echo "%%~dA%%~pA%docname%"
    echo "%%~dA%%~pA%docname%" > "%%~dA%%~pA%docname%"
    )
pause
```