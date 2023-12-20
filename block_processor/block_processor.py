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
logger = logging.getLogger("block_processor")
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


async def block_processor():
    """
    Main function to process blocks.
    """
    logger.info("Starting block processor service")
    connection = None
    try:
        # Setup RabbitMQ connection and channel
        logger.info("Connecting to RabbitMQ...")
        connection = await connect_to_rabbitmq()
        channel = await connection.channel()

        # Ensure the queue exists
        logger.info(f"Declaring RabbitMQ queue: {QUEUE_NAME}")
        queue = await channel.declare_queue(QUEUE_NAME, durable=True)

        # Consume messages from RabbitMQ
        async with queue.iterator() as queue_iter:
            async for message in queue_iter:
                async with message.process():
                    block_number = message.body.decode()
                    logger.info(f"Received block number: {block_number}")

    except Exception as e:
        logger.error(f"Error in block processor service: {e}")
        logger.error(traceback.format_exc()) 

    finally:
        if connection and not connection.is_closed:
            logger.info("Closing RabbitMQ connection")
            await connection.close()
        logger.info("Block processor service stopped")


if __name__ == "__main__":
    asyncio.run(block_processor())








# import asyncio
# import logging
# import os
# import aio_pika

# # Configure logging
# logger = logging.getLogger("block_processor")
# logger.setLevel(logging.INFO)
# handler = logging.StreamHandler()
# formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
# handler.setFormatter(formatter)
# logger.addHandler(handler)

# # Environment Variables
# RABBITMQ_URL = f"amqp://{os.getenv('RABBITMQ_DEFAULT_USER')}:{os.getenv('RABBITMQ_DEFAULT_PASS')}@{os.getenv('RABBITMQ_HOST')}:{os.getenv('RABBITMQ_PORT')}/"
# QUEUE_NAME = os.getenv('RABBITMQ_QUEUE_NAME')

# async def connect_to_rabbitmq():
#     """
#     Attempt to connect to RabbitMQ with retries using exponential backoff.
#     """
#     initial_delay = 10  # seconds, initial delay before the first connection attempt
#     max_retries = 5
#     retry_delay = 5  # seconds, initial delay between retries

#     await asyncio.sleep(initial_delay)  # Wait for RabbitMQ to start up

#     for attempt in range(1, max_retries + 1):
#         try:
#             connection = await aio_pika.connect_robust(RABBITMQ_URL)
#             return connection
#         except Exception as e:
#             logger.error(f"Attempt {attempt} failed to connect to RabbitMQ: {e}")
#             if attempt < max_retries:
#                 wait_time = retry_delay * (2 ** (attempt - 1))  # Exponential backoff
#                 logger.info(f"Retrying in {wait_time} seconds...")
#                 await asyncio.sleep(wait_time)
#     raise Exception("Failed to connect to RabbitMQ after several attempts")


# async def block_processor():
#     """
#     Main function to process blocks.
#     """
#     logger.info("Starting block processor service")
#     try:
#         # Setup RabbitMQ connection and channel
#         logger.info("Connecting to RabbitMQ...")
#         connection = await connect_to_rabbitmq()
#         channel = await connection.channel()

#         # Ensure the queue exists
#         logger.info(f"Declaring RabbitMQ queue: {QUEUE_NAME}")
#         queue = await channel.declare_queue(QUEUE_NAME, durable=True)

#         # Consume messages from RabbitMQ
#         async with queue.iterator() as queue_iter:
#             async for message in queue_iter:
#                 with message.process():
#                     block_number = message.body.decode()
#                     logger.info(f"Received block number: {block_number}")

#     except Exception as e:
#         logger.error(f"Error in block processor service: {e}")
#     finally:
#         if connection and not connection.is_closed:
#             logger.info("Closing RabbitMQ connection")
#             await connection.close()
#         logger.info("Block processor service stopped")



# if __name__ == "__main__":
#     asyncio.run(block_processor())
