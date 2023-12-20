import asyncio
from web3 import AsyncWeb3, Web3
from web3.providers import WebsocketProviderV2
import aio_pika
import json
import logging
import os
import time
import traceback

# Configure logging
logger = logging.getLogger("block_checker")
logger.setLevel(logging.DEBUG)
handler = logging.StreamHandler()
formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
handler.setFormatter(formatter)
logger.addHandler(handler)

# Access environment variables
RABBITMQ_URL = f"amqp://{os.getenv('RABBITMQ_DEFAULT_USER')}:{os.getenv('RABBITMQ_DEFAULT_PASS')}@{os.getenv('RABBITMQ_HOST')}:{os.getenv('RABBITMQ_PORT')}/"
QUEUE_NAME = os.getenv('RABBITMQ_QUEUE_NAME')


async def connect_to_rabbitmq():
    """
    Attempt to connect to RabbitMQ with retries using exponential backoff.
    """
    max_retries = 5
    retry_delay = 5  # seconds, initial delay between retries

    for attempt in range(1, max_retries + 1):
        try:
            connection = await aio_pika.connect_robust(RABBITMQ_URL)
            return connection
        except Exception as e:
            logger.error(f"Attempt {attempt} failed to connect to RabbitMQ: {e}")
            if attempt < max_retries:
                wait_time = retry_delay * (2 ** (attempt - 1))  # Exponential backoff
                logger.info(f"Retrying in {wait_time} seconds...")
                await asyncio.sleep(wait_time)
    raise Exception("Failed to connect to RabbitMQ after several attempts")


async def send_to_rabbitmq(block_number, channel):
    """
    Send the block number to RabbitMQ.
    """
    try:
        # Convert block number to string and encode to bytes
        message_body = str(block_number).encode()

        # Publish message
        await channel.default_exchange.publish(
            aio_pika.Message(body=message_body),
            routing_key=QUEUE_NAME
        )
        logger.info(f"Sent block number {block_number} to RabbitMQ")
    except Exception as e:
        logger.error(f"Error sending to RabbitMQ: {e}")
        logger.error(traceback.format_exc())



async def block_checker():
    logger.info("Starting block checker service")
    try:
        # Setup RabbitMQ connection and channel with retries
        logger.info("Connecting to RabbitMQ...")
        connection = await connect_to_rabbitmq()
        channel = await connection.channel()

        # Ensure the queue exists
        logger.info(f"Declaring RabbitMQ queue: {QUEUE_NAME}")
        await channel.declare_queue(QUEUE_NAME, durable=True)

        async for w3 in AsyncWeb3.persistent_websocket(WebsocketProviderV2("wss://ethereum.publicnode.com")):
            logger.info("Connected to blockchain WebSocket")
            try:
                logger.info("Subscribing to new blockchain block headers")
                subscription_id = await w3.eth.subscribe("newHeads")
                async for block in w3.ws.listen_to_websocket():
                    block_number = int(block['result']['number'], 16)
                    logger.info(f"New block received: Number {block_number}")
                    await send_to_rabbitmq(block_number, channel)
            except Exception as e:
                logger.error(f"Error in blockchain subscription: {e}")
                continue  # Reconnect on error

    except Exception as e:
        logger.error(f"Error setting up RabbitMQ: {e}")
    finally:
        # Close the RabbitMQ connection
        if 'connection' in locals():
            logger.info("Closing RabbitMQ connection")
            await connection.close()
        logger.info("Block checker service stopped")

if __name__ == "__main__":
    asyncio.run(block_checker())
