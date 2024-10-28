from hexbytes import HexBytes

def hex_to_str(hex_value: HexBytes) -> str:
    # Ensure input is HexBytes type
    if not isinstance(hex_value, HexBytes):
        raise TypeError(f"Expected HexBytes, got {type(hex_value)}")
    
    # Handle None or empty input
    if hex_value is None:
        return None
    
    # Convert to hex string, maintaining '0x' prefix
    return '0x' + hex_value.hex()
    