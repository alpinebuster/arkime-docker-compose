import sys
from src.util.rust import check_rust_lib_up_to_date

# Check that we're not running on an unsupported Python version.
#
# Note that we use an (unneeded) variable here so that pyupgrade doesn't nuke the
# if-statement completely.
py_version = sys.version_info
if py_version < (3, 8):
    print("Backbone_td requires Python 3.8 or above.")
    sys.exit(1)

check_rust_lib_up_to_date()
