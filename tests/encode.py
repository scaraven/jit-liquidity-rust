import os
from dotenv import load_dotenv
from uniswap_universal_router_decoder import FunctionRecipient, RouterCodec

from web3 import Web3
from web3.middleware import ExtraDataToPOAMiddleware
from web3.types import Wei
import datetime

# -----------------------
# Load environment variables
# -----------------------
load_dotenv()
PRIVATE_KEY = os.getenv("PRIVATE_KEY")
RPC_URL = os.getenv("SEPOLIA_RPC_URL")

if not PRIVATE_KEY or not RPC_URL:
    raise ValueError("Missing PRIVATE_KEY or SEPOLIA_RPC_URL in .env file")


# -----------------------
# Initialize Web3
# -----------------------
w3 = Web3(Web3.HTTPProvider(RPC_URL))
# Inject the POA middleware if on a chain like Sepolia, Goerli, etc.
w3.middleware_onion.inject(ExtraDataToPOAMiddleware, layer=0)
account = w3.eth.account.from_key(PRIVATE_KEY)


# -----------------------
# Define Addresses & ABIs
# -----------------------
weth_sepolia = w3.to_checksum_address("0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14")
usdt_sepolia = w3.to_checksum_address("0xaA8E23Fb1079EA71e0a56F48a2aA51851D8433D0")
universal_router_addr = w3.to_checksum_address("0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD")

# -----------------------
# Helper: Dynamic Fee Params
# -----------------------
def get_dynamic_fee_params(w3, priority_gwei="1", multiplier=1.2):
    """
    Returns a dict with EIP-1559 fee parameters:
    - maxPriorityFeePerGas (tip) set to `priority_gwei` Gwei
    - maxFeePerGas set to ~ (multiplier * baseFee) + tip
    """
    latest_block = w3.eth.get_block("pending")
    base_fee = latest_block.get("baseFeePerGas", 0)  # baseFeePerGas in Wei
    # If baseFee is missing or zero, fallback to w3.eth.gas_price
    if base_fee == 0:
        fallback_gas_price = w3.eth.gas_price
        print(f"Warning: 'baseFeePerGas' not available, using fallback gasPrice = {fallback_gas_price}")
        return {"gasPrice": fallback_gas_price}

    # Convert priority fee from Gwei to Wei
    priority_fee = w3.to_wei(priority_gwei, "gwei")

    max_fee_per_gas = int(base_fee * multiplier) + priority_fee

    return {
        "maxFeePerGas": max_fee_per_gas,
        "maxPriorityFeePerGas": priority_fee
    }


# -----------------------
# Step 1: Transaction Calculations
# -----------------------
amount_in = int(Web3.to_wei('0.00001', 'ether'))
gas_params = get_dynamic_fee_params(w3, priority_gwei="1", multiplier=1.2)

# -----------------------
# Step 2: Encode Swap Data
# -----------------------
codec = RouterCodec()
expiry = int(datetime.datetime.now().timestamp()) + 1000  # expire in ~1000s

encoded_data = (
    codec.encode
    .chain()
    .wrap_eth(
        FunctionRecipient.ROUTER,
        amount_in,
    )
    .v3_swap_exact_in(
        FunctionRecipient.SENDER,
        amount_in,
        Wei(0),
        [
            weth_sepolia,
            3000,        # fee tier
            usdt_sepolia
        ],
        payer_is_sender=False
    )
    .build(expiry)
)

print(f"🔹 Encoded swap data: {encoded_data}")


# -----------------------
# Step 4: Execute Swap (WETH -> USDC)
# -----------------------
execute_tx = {
    "from": account.address,
    "nonce": w3.eth.get_transaction_count(account.address, 'pending'),
    "to": universal_router_addr,
    "data": encoded_data,
    "gas": 500000,  # Increase if you suspect a higher gas usage
    "value": amount_in,
    "chainId": w3.eth.chain_id,
    **gas_params
}

signed_execute_tx = w3.eth.account.sign_transaction(execute_tx, private_key=PRIVATE_KEY)
execute_tx_hash = w3.eth.send_raw_transaction(signed_execute_tx.raw_transaction)
print(f"🚀 Executing swap WETH -> USDC. Tx hash: {execute_tx_hash.hex()}")
execute_receipt = w3.eth.wait_for_transaction_receipt(execute_tx_hash)
print(f"   Mined in block {execute_receipt.blockNumber} with status {execute_receipt.status}")
