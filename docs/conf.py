# -- Project information -----------------------------------------------------

project = "HubSpot Emulator"
author = "Morgan West"

# Optional but recommended
release = "latest"


# -- General configuration ---------------------------------------------------

extensions = [
    "myst_parser",
]

myst_enable_extensions = [
    "colon_fence",
]

templates_path = ["_templates"]

exclude_patterns = []


# -- HTML output -------------------------------------------------------------

html_theme = "furo"

html_static_path = ["_static"]

html_theme_options = {
    "source_repository": "https://github.com/morganzwest/hsemulator",
    "source_branch": "main",
    "source_directory": "docs/",
}
