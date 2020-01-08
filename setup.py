__author__                      = "Perry Kundert"
__email__                       = "perry.kundert@holo.host"
__copyright__                   = "Copyright (c) 2020 Holo Limited, Gibralter"
__license__                     = "Apache License, Version 2.0"

from setuptools import setup
import os

name			= 'hpos-config'
package			= 'hpos_config'

here			= os.path.abspath( os.path.dirname( __file__ ))

with open( os.path.join( here, package, 'version.py' ), 'r' ) as version:
    exec( version.read() )

install_requires	= []
requirements_txt	= os.path.join( here, "requirements.txt" )
if os.path.exists( requirements_txt ):
    with open( requirements_txt, 'r' ) as reqirements:
        install_requires = requirements.readlines()

setup(
    name		= name,
    version		= __version__,
    packages		= [ package ],
    tests_require	= [ 'pytest' ],
    install_requires	= install_requires,
    author		= "Perry Kundert",
    author_email	= "perry.kundert@holo.host",
    description		= "Schema validation for HPOS Config",
    license		= "Apache License, Version 2.0",
    keywords		= "HPOS config schema validation",
    url			= "https://github.com/Holo-Host/hpos-config",
    classifiers		= [
        "License :: OSI Approved :: Apache Software License",
        "Programming Language :: Python :: 3.7",
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
    ],
    long_description		= """\
The hpos_config Python module provides access to various operations on the
`hpos-config.json` file to Python 3.7+ programs.

PACKAGES

  .schema  -- validate the data schema on a valid `hpos-config.json` file
    Only validates the portions required by HPOS, and only at a superficial level; 
    presence, and basic data type.

"""
)
