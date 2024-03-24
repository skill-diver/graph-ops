Building Python Docs
====================

Ofnil uses Sphinx for Python documentation.

```bash
pip install sphinx
pip install sphinx-rtd-theme
cd $OFNIL_HOME/docs/python
make html
# The html root folder is $OFNIL_HOME/docs/python/build/html
# You may use the Python HTTP server to host the documentation locally at port 8080 (change the port if needed):
python3 -m http.server 8080 --directory build/html
```
