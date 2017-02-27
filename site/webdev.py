#!/bin/python3

import os

from flask import Flask, render_template, request, Markup
from runcode import runcode
app = Flask(__name__)

default = """x = 9

if x:
    x = 'string'
"""

examples = os.listdir(os.path.join('..', 'examples'))
examples.sort()

@app.route("/", methods=['POST', 'GET'])
def run():
    if request.method == 'POST':
        code = request.form['code']
        code = code.strip()
        forsite = runcode.Forsite(code)
        result = forsite.run()
        result = Markup(result)
    else:
        name = request.args.get('code')
        try:
          with open(os.path.join('..', 'examples', name)) as f:
            code = f.read()
        except:
          code = default
        result = ''
    return render_template("main.html",
                           code=code,
                           result=result,
                           examples=examples)

if __name__ == "__main__":
    app.run()
