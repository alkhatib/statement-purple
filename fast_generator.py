#!/usr/bin/env python3
"""
Fast Transaction Test Data Generator

Generates CSV files quickly with random data - no realistic constraints.
Perfect for performance testing and large dataset generation.
"""

import csv
import random
import argparse
from typing import List

def generate_amount() -> str:
    """Generate a random amount as string with up to 4 decimal places"""
    # Random amount between 0.01 and 10000.00
    amount = random.uniform(0.01, 10000.0)
    # Format to 1-4 decimal places randomly for variety
    decimals = random.choice([1, 2, 3, 4])
    return f"{amount:.{decimals}f}"

def generate_transaction_batch(batch_size: int, num_clients: int, 
                             deposit_pct: float, withdrawal_pct: float, 
                             dispute_pct: float, resolve_pct: float, 
                             chargeback_pct: float, next_tx_id: int) -> List[List[str]]:
    """Generate a batch of transactions as CSV rows"""
    batch = []
    
    # Calculate counts for this batch
    deposits = int(batch_size * deposit_pct)
    withdrawals = int(batch_size * withdrawal_pct)
    disputes = int(batch_size * dispute_pct)
    resolves = int(batch_size * resolve_pct)
    chargebacks = int(batch_size * chargeback_pct)
    
    tx_id = next_tx_id
    
    # Generate deposits
    for _ in range(deposits):
        client = random.randint(1, num_clients)
        amount = generate_amount()
        batch.append(["deposit", str(client), str(tx_id), amount])
        tx_id += 1
    
    # Generate withdrawals
    for _ in range(withdrawals):
        client = random.randint(1, num_clients)
        amount = generate_amount()
        batch.append(["withdrawal", str(client), str(tx_id), amount])
        tx_id += 1
    
    # Generate disputes (reference random tx_ids)
    for _ in range(disputes):
        client = random.randint(1, num_clients)
        # Reference a random transaction ID that might exist
        ref_tx_id = random.randint(1, max(1, tx_id - 1))
        batch.append(["dispute", str(client), str(ref_tx_id), ""])
    
    # Generate resolves
    for _ in range(resolves):
        client = random.randint(1, num_clients)
        ref_tx_id = random.randint(1, max(1, tx_id - 1))
        batch.append(["resolve", str(client), str(ref_tx_id), ""])
    
    # Generate chargebacks
    for _ in range(chargebacks):
        client = random.randint(1, num_clients)
        ref_tx_id = random.randint(1, max(1, tx_id - 1))
        batch.append(["chargeback", str(client), str(ref_tx_id), ""])
    
    # Shuffle the batch
    random.shuffle(batch)
    
    return batch, tx_id

def generate_fast_csv(filename: str, num_clients: int, total_transactions: int,
                     deposit_pct: float = 0.6, withdrawal_pct: float = 0.3,
                     dispute_pct: float = 0.05, resolve_pct: float = 0.03,
                     chargeback_pct: float = 0.02, batch_size: int = 10000,
                     seed: int = 42):
    """Generate CSV file in batches for memory efficiency"""
    
    random.seed(seed)
    
    print(f"Generating {total_transactions:,} transactions for {num_clients:,} clients...")
    print(f"Writing to: {filename}")
    
    with open(filename, 'w', newline='') as csvfile:
        writer = csv.writer(csvfile)
        
        # Write header
        writer.writerow(['type', 'client', 'tx', 'amount'])
        
        transactions_written = 0
        next_tx_id = 1
        
        # Process in batches
        while transactions_written < total_transactions:
            remaining = total_transactions - transactions_written
            current_batch_size = min(batch_size, remaining)
            
            # Generate batch
            batch, next_tx_id = generate_transaction_batch(
                current_batch_size, num_clients, deposit_pct, withdrawal_pct,
                dispute_pct, resolve_pct, chargeback_pct, next_tx_id
            )
            
            # Write batch
            writer.writerows(batch)
            transactions_written += len(batch)
            
            # Progress update
            if transactions_written % 100000 == 0:
                print(f"  Generated {transactions_written:,} transactions...")
    
    print(f"✅ Generated {transactions_written:,} transactions in {filename}")

def generate_simple_csv(filename: str, num_clients: int, total_transactions: int,
                       seed: int = 42):
    """Generate very simple CSV with just deposits and withdrawals"""
    random.seed(seed)
    
    print(f"Generating {total_transactions:,} simple transactions...")
    
    with open(filename, 'w', newline='') as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(['type', 'client', 'tx', 'amount'])
        
        for tx_id in range(1, total_transactions + 1):
            client = random.randint(1, num_clients)
            tx_type = random.choice(["deposit", "withdrawal"])
            amount = generate_amount()
            
            writer.writerow([tx_type, str(client), str(tx_id), amount])
            
            if tx_id % 100000 == 0:
                print(f"  Generated {tx_id:,} transactions...")
    
    print(f"✅ Generated {total_transactions:,} simple transactions in {filename}")

def main():
    parser = argparse.ArgumentParser(description='Fast transaction data generator')
    
    # Basic options
    parser.add_argument('--clients', type=int, default=1000,
                      help='Number of unique clients (default: 1000)')
    parser.add_argument('--transactions', type=int, default=100000,
                      help='Total number of transactions (default: 100000)')
    parser.add_argument('--output', type=str, default='fast_test.csv',
                      help='Output CSV filename (default: fast_test.csv)')
    parser.add_argument('--seed', type=int, default=42,
                      help='Random seed (default: 42)')
    
    # Mode selection
    parser.add_argument('--simple', action='store_true',
                      help='Generate only deposits/withdrawals (fastest)')
    
    # Transaction percentages (for complex mode)
    parser.add_argument('--deposit-pct', type=float, default=0.6,
                      help='Percentage of deposits (default: 0.6)')
    parser.add_argument('--withdrawal-pct', type=float, default=0.3,
                      help='Percentage of withdrawals (default: 0.3)')
    parser.add_argument('--dispute-pct', type=float, default=0.05,
                      help='Percentage of disputes (default: 0.05)')
    parser.add_argument('--resolve-pct', type=float, default=0.03,
                      help='Percentage of resolves (default: 0.03)')
    parser.add_argument('--chargeback-pct', type=float, default=0.02,
                      help='Percentage of chargebacks (default: 0.02)')
    
    # Performance options
    parser.add_argument('--batch-size', type=int, default=10000,
                      help='Batch size for processing (default: 10000)')
    
    args = parser.parse_args()
    
    if args.simple:
        generate_simple_csv(args.output, args.clients, args.transactions, args.seed)
    else:
        generate_fast_csv(
            args.output, args.clients, args.transactions,
            args.deposit_pct, args.withdrawal_pct, args.dispute_pct,
            args.resolve_pct, args.chargeback_pct, args.batch_size, args.seed
        )

if __name__ == '__main__':
    main()
