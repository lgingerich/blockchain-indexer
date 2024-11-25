import asyncio
from web3 import AsyncWeb3
from web3.providers import WebSocketProvider

# Connect to a WebSocket provider
# WSS_URL = "wss://ws.zkevm.cronos.org"
# WSS_URL = "wss://arbitrum-one.publicnode.com"
WSS_URL = "wss://mainnet.era.zksync.io/ws"

async def subscribe_to_blocks():
    async with AsyncWeb3(WebSocketProvider(WSS_URL)) as w3:
        subscription_id = await w3.eth.subscribe("newHeads")
        
        try:
            async for response in w3.socket.process_subscriptions():
                block_number = response['result']['number']
                print(f"New block received: {block_number}")
                # print(response)
        except Exception as e:
            print(f"An error occurred: {e}")
        finally:
            await w3.eth.unsubscribe(subscription_id)

if __name__ == "__main__":
    asyncio.run(subscribe_to_blocks())
