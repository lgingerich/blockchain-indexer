from sqlalchemy.orm import Session
from .schema import Block, Transaction, Log

class BlockRepository:
    @staticmethod
    def create_block(session: Session, block_data: dict) -> Block:
        block = Block(**block_data)
        session.add(block)
        return block

    @staticmethod
    def get_block_by_number(session: Session, block_number: int) -> Block:
        return session.query(Block).filter(Block.number == block_number).first()

    @staticmethod
    def get_blocks(session: Session, skip: int = 0, limit: int = 100) -> list[Block]:
        return session.query(Block).offset(skip).limit(limit).all()
    
    @staticmethod
    def get_latest_block(session: Session) -> Block:
        """
        Get the latest block from the database.
        """
        return session.query(Block).order_by(Block.number.desc()).first()

class TransactionRepository:
    @staticmethod
    def create_transaction(session: Session, transaction_data: dict) -> Transaction:
        transaction = Transaction(**transaction_data)
        session.add(transaction)
        return transaction

    @staticmethod
    def get_transaction_by_hash(session: Session, transaction_hash: str) -> Transaction:
        return session.query(Transaction).filter(Transaction.hash == transaction_hash).first()

    @staticmethod
    def get_transactions(session: Session, skip: int = 0, limit: int = 100) -> list[Transaction]:
        return session.query(Transaction).offset(skip).limit(limit).all()

class LogRepository:
    @staticmethod
    def create_log(session: Session, log_data: dict) -> Log:
        log = Log(**log_data)
        session.add(log)
        return log

    @staticmethod
    def get_logs_by_transaction_hash(session: Session, transaction_hash: str) -> list[Log]:
        return session.query(Log).filter(Log.transaction_hash == transaction_hash).all()

    @staticmethod
    def get_logs(session: Session, skip: int = 0, limit: int = 100) -> list[Log]:
        return session.query(Log).offset(skip).limit(limit).all()

# Additional functions to update or delete records could also be added here as needed.

def save_changes(session: Session):
    session.commit()
