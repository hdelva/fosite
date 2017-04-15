#!/bin/bash

#pandoc -i appendix.md -o appendix.tex
#pandoc report.md --filter=pandoc-citeproc --biblio=biblio.bib --template=template.tex --number-sections --csl=style.csl --latex-engine=xelatex -o report.pdf

pandoc report.md --filter=pandoc-citeproc --biblio=biblio.bib --template=template.tex --number-sections --csl=style.csl -o report.tex

xelatex -shell-escape report.tex
