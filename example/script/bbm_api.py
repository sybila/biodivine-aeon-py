import sys

from biodivine_aeon import *

# This script demonstrates how to use the Biodivine Boolean Models (BBM) API
# to retrieve and work with models from the BBM database.
#
# You can fetch models using their numeric IDs (as strings). The IDs are
# consecutive integers, and the database contains at least 200 models.
#
# Basic usage - fetch a model and display its information:
# ```
# python3 bbm_api.py 1
# ```
#
# Fetch a model and convert it to a BooleanNetwork:
# ```
# python3 bbm_api.py 1 --network
# ```
#
# Fetch multiple models:
# ```
# python3 bbm_api.py 1 5 10
# ```

if len(sys.argv) < 2:
    print("Usage: python3 bbm_api.py <model_id> [<model_id> ...] [--network]")
    print("Example: python3 bbm_api.py 1")
    print("Example: python3 bbm_api.py 1 5 10 --network")
    sys.exit(1)

# Check if --network flag is present
show_network = "--network" in sys.argv
model_ids = [arg for arg in sys.argv[1:] if arg != "--network"]

print(f"Fetching {len(model_ids)} model(s) from BBM database...")
print()

for model_id in model_ids:
    try:
        # Fetch the model from BBM database
        model = BiodivineBooleanModels.fetch_model(model_id)
        
        print(f"Model ID: {model_id}")
        print(f"  Database ID: {model.id}")
        print(f"  Name: {model.name}")
        print(f"  Variables: {model.variables}")
        print(f"  Inputs: {model.inputs}")
        print(f"  Regulations: {model.regulations}")
        
        if model.keywords:
            print(f"  Keywords: {', '.join(model.keywords)}")
        
        if model.url_publication:
            print(f"  Publication: {model.url_publication}")
        
        if model.url_model:
            print(f"  Model URL: {model.url_model}")
        
        # If --network flag is set, convert to BooleanNetwork and show some info
        if show_network:
            print()
            print("  Converting to BooleanNetwork...")
            bn = model.to_bn_default()
            print(f"  Network variables: {bn.variable_count()}")
            print(f"  Network regulations: {bn.regulation_count()}")
            print(f"  Variable names: {', '.join(bn.variable_names())}")
        
        print()
        
    except Exception as e:
        print(f"Error fetching model {model_id}: {e}")
        print()

