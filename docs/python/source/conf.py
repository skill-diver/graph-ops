# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html


import datetime
import os.path as osp
import sys

import ofnil

project_path = osp.join(osp.dirname((osp.dirname((osp.dirname(osp.dirname(__file__)))))), "python")
sys.path.insert(0, project_path)  # add ofnil python project path
sys.path.append(osp.dirname(__file__))  # add path for patch extension

# -- Project information -----------------------------------------------------

project = "ofnil"
author = "Ofnil Team"
copyright = f"{datetime.datetime.now().year}, {author}"
release = ofnil.__version__

# -- General configuration ---------------------------------------------------

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.autosummary",
    "sphinx.ext.napoleon",
    "sphinx.ext.viewcode",
    "patch",
]

templates_path = ["_templates"]
exclude_patterns = []

# -- Options for HTML output -------------------------------------------------

html_theme = "sphinx_rtd_theme"
html_static_path = ["_static"]
html_css_files = ["css/ofnil.css"]

# -- Options for extensions --------------------------------------------------

add_module_names = False
autodoc_member_order = "bysource"


def setup(app):
    def rst_jinja_render(app, _, source):
        rst_context = {"ofnil": ofnil}
        source[0] = app.builder.templates.render_string(source[0], rst_context)

    app.connect("source-read", rst_jinja_render)
