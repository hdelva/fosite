#!/bin/bash

#pandoc -i appendix.md -o appendix.tex
#pandoc report.md --filter=pandoc-citeproc --biblio=biblio.bib --template=template.tex --number-sections --csl=style.csl --latex-engine=xelatex -o report.pdf

pandoc report.md --top-level-division=chapter --filter=pandoc-citeproc --biblio=biblio.bib --template=template.tex --number-sections --csl=style.csl  -o report.tex
#pandoc report.md --top-level-division=chapter --filter=pandoc-citeproc --template=template.tex --number-sections  -o report.tex

xelatex -shell-escape report.tex
