from sqlalchemy import Column, Integer, String, Numeric, BigInteger, ForeignKey, Text, LargeBinary, DateTime
from sqlalchemy.orm import relationship
from sqlalchemy.ext.declarative import declarative_base

Base = declarative_base()

class Block(Base):
    __tablename__ = 'blocks'
    
    number = Column(BigInteger, primary_key=True)
    hash = Column(String(66), nullable=False, unique=True)
    parent_hash = Column(String(66), nullable=False)
    nonce = Column(LargeBinary(8))  # 8 bytes
    sha3_uncles = Column(String(66), nullable=False)
    transactions_root = Column(String(66), nullable=False)
    state_root = Column(String(66), nullable=False)
    receipts_root = Column(String(66), nullable=False)
    miner = Column(String(42), nullable=False)
    difficulty = Column(Numeric, nullable=False)
    total_difficulty = Column(Numeric, nullable=False)
    extra_data = Column(Text)
    size = Column(Integer, nullable=False)
    gas_limit = Column(BigInteger, nullable=False)
    gas_used = Column(BigInteger, nullable=False)
    timestamp = Column(DateTime, nullable=False)
    transactions = relationship("Transaction", back_populates="block")

class Transaction(Base):
    __tablename__ = 'transactions'
    
    hash = Column(String(66), primary_key=True)
    block_number = Column(BigInteger, ForeignKey('blocks.number'))
    from_address = Column(String(42), nullable=False)
    to_address = Column(String(42), nullable=True)
    value = Column(Numeric(32), nullable=False)
    gas = Column(BigInteger, nullable=False)
    gas_price = Column(BigInteger, nullable=False)
    nonce = Column(BigInteger, nullable=False)
    transaction_index = Column(Integer, nullable=False)
    input = Column(Text)
    v = Column(String(4))
    r = Column(String(66))
    s = Column(String(66))
    block = relationship("Block", back_populates="transactions")
    logs = relationship("Log", back_populates="transaction")

class Log(Base):
    __tablename__ = 'logs'
    
    id = Column(Integer, primary_key=True, autoincrement=True)
    log_index = Column(Integer, nullable=False)
    transaction_hash = Column(String(66), ForeignKey('transactions.hash'))
    block_number = Column(BigInteger, ForeignKey('blocks.number'))
    address = Column(String(42), nullable=False)
    data = Column(Text)
    topics = Column(Text)  # This could be an array of Strings, depending on how you want to model it
    transaction = relationship("Transaction", back_populates="logs")
