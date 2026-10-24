#!/usr/bin/env python3
"""Compatibility entry point: deploy Rust + React, preserving configuration/data."""
import os
from pathlib import Path
import sys
script = Path(__file__).resolve().with_name('deploy_platform.py')
os.execv(sys.executable, [sys.executable, str(script), *sys.argv[1:]])
