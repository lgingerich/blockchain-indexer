from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.engine.reflection import Inspector
import logging
from db.schema import Base

logger = logging.getLogger(__name__)

# DuckDB connection URI
# In-memory database: 'duckdb:///:memory:'
# On-disk database: 'duckdb:///path_to_your_database_file'
DATABASE_URI = 'duckdb:///:memory:'

engine = create_engine(DATABASE_URI)
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

Base = declarative_base()

def table_exists(engine, name):
    """
    Check whether a table with the given name exists.
    """
    inspector = Inspector.from_engine(engine)
    return name in inspector.get_table_names()

def init_db():
    """
    Create the database tables if they don't already exist.
    """
    logger.info("init_db() called")
    if not table_exists(engine, 'blocks') or \
       not table_exists(engine, 'transactions') or \
       not table_exists(engine, 'logs'):
        Base.metadata.create_all(bind=engine)
        logger.info("Successfully created blocks, transactions, and logs tables.")
    else:
        logger.info("All tables already exist.")

def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
