import asyncio
import os
import aio_pika
import logging

logger = logging.getLogger(__name__)

RABBITMQ_URL = os.getenv('RABBITMQ_URL')
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
            logger.info(f"Connected to RabbitMQ on attempt {attempt}.")
            return connection
        except Exception as e:
            logger.error(f"Attempt {attempt} failed to connect to RabbitMQ: {e}")
            if attempt < max_retries:
                wait_time = retry_delay * (2 ** (attempt - 1))  # Exponential backoff
                logger.info(f"Retrying in {wait_time} seconds...")
                await asyncio.sleep(wait_time)
    raise Exception("Failed to connect to RabbitMQ after several attempts")

async def message_handler(message: aio_pika.IncomingMessage, latest_block_queue: asyncio.Queue):
    """
    Handle incoming messages from RabbitMQ.
    """
    async with message.process():
        block_number = message.body.decode()
        logger.info(f"Received block number: {block_number}")
        await latest_block_queue.put(block_number)

async def consume_blocks():
    """
    Consume messages from RabbitMQ and return the latest block number.
    """
    logger.info("Starting RabbitMQ consumer service.")
    connection = await connect_to_rabbitmq()
    latest_block_queue = asyncio.Queue()

    async with connection:
        # Create a channel
        channel = await connection.channel()
        
        # Declare the queue
        queue = await channel.declare_queue(QUEUE_NAME, durable=True)
        
        # Start consuming messages
        await queue.consume(lambda message: message_handler(message, latest_block_queue))

        logger.info(f"Consuming messages from queue: {QUEUE_NAME}")

        # Return the latest block number from the queue
        return await latest_block_queue.get()