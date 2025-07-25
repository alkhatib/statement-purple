#!/usr/bin/env python3
"""
Transaction Test Data Generator for Rust Payments Engine

Generates CSV files with configurable transaction patterns for testing
the payments engine implementation.
"""

import csv
import random
import argparse
from decimal import Decimal, ROUND_HALF_UP
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass
from enum import Enum

class TransactionType(Enum):
    DEPOSIT = "deposit"
    WITHDRAWAL = "withdrawal"
    DISPUTE = "dispute"
    RESOLVE = "resolve"
    CHARGEBACK = "chargeback"

@dataclass
class Transaction:
    tx_type: str
    client: int
    tx_id: int
    amount: Optional[Decimal] = None
    
    def to_csv_row(self) -> List[str]:
        """Convert transaction to CSV row format"""
        if self.amount is not None:
            # Format to 4 decimal places as specified
            amount_str = f"{self.amount:.4f}"
        else:
            amount_str = ""
        return [self.tx_type, str(self.client), str(self.tx_id), amount_str]

class TransactionGenerator:
    def __init__(self, num_clients: int, total_transactions: int, 
                 deposit_pct: float = 0.6, withdrawal_pct: float = 0.3,
                 dispute_pct: float = 0.08, resolve_pct: float = 0.6,
                 chargeback_pct: float = 0.3, seed: Optional[int] = None):
        """
        Initialize transaction generator
        
        Args:
            num_clients: Number of unique clients (1 to num_clients)
            total_transactions: Total number of transactions to generate
            deposit_pct: Percentage of transactions that are deposits (0.0-1.0)
            withdrawal_pct: Percentage of transactions that are withdrawals (0.0-1.0)
            dispute_pct: Percentage of deposits that get disputed (0.0-1.0)
            resolve_pct: Percentage of disputes that get resolved (0.0-1.0)
            chargeback_pct: Percentage of disputes that get charged back (0.0-1.0)
            seed: Random seed for reproducible results
        """
        self.num_clients = num_clients
        self.total_transactions = total_transactions
        self.deposit_pct = deposit_pct
        self.withdrawal_pct = withdrawal_pct
        self.dispute_pct = dispute_pct
        self.resolve_pct = resolve_pct
        self.chargeback_pct = chargeback_pct
        
        if seed is not None:
            random.seed(seed)
        
        # Validate percentages
        if deposit_pct + withdrawal_pct > 1.0:
            raise ValueError("deposit_pct + withdrawal_pct cannot exceed 1.0")
        
        if resolve_pct + chargeback_pct > 1.0:
            raise ValueError("resolve_pct + chargeback_pct cannot exceed 1.0")
        
        # Track state for generating realistic transactions
        self.client_balances: Dict[int, Decimal] = {}
        self.disputable_transactions: Dict[int, Decimal] = {}  # tx_id -> amount
        self.disputed_transactions: Dict[int, Decimal] = {}   # tx_id -> amount
        self.transactions: List[Transaction] = []
        self.next_tx_id = 1
    
    def generate_amount(self, min_amount: float = 0.01, max_amount: float = 10000.0) -> Decimal:
        """Generate a random transaction amount with realistic distribution"""
        # Use log-normal distribution for more realistic amounts
        # Most transactions small, some large ones
        amount = random.lognormvariate(3.0, 1.5)  # Mean around $20, some very large
        amount = max(min_amount, min(amount, max_amount))
        
        # Round to 4 decimal places as specified
        return Decimal(str(amount)).quantize(Decimal('0.0001'), rounding=ROUND_HALF_UP)
    
    def get_client_balance(self, client_id: int) -> Decimal:
        """Get current available balance for a client"""
        return self.client_balances.get(client_id, Decimal('0'))
    
    def update_balance(self, client_id: int, amount: Decimal):
        """Update client balance"""
        current = self.client_balances.get(client_id, Decimal('0'))
        self.client_balances[client_id] = current + amount
    
    def generate_deposit(self, client_id: int) -> Transaction:
        """Generate a deposit transaction"""
        amount = self.generate_amount()
        tx_id = self.next_tx_id
        self.next_tx_id += 1
        
        # Update balance and track as disputable
        self.update_balance(client_id, amount)
        self.disputable_transactions[tx_id] = amount
        
        return Transaction(
            tx_type=TransactionType.DEPOSIT.value,
            client=client_id,
            tx_id=tx_id,
            amount=amount
        )
    
    def generate_withdrawal(self, client_id: int) -> Optional[Transaction]:
        """Generate a withdrawal transaction (only if sufficient funds)"""
        balance = self.get_client_balance(client_id)
        
        if balance <= Decimal('0.01'):
            return None  # Skip withdrawal if insufficient funds
        
        # Withdraw up to 80% of available balance
        max_withdrawal = balance * Decimal('0.8')
        amount = self.generate_amount(max_amount=float(max_withdrawal))
        amount = Decimal(str(amount))
        
        if amount > balance:
            return None
        
        tx_id = self.next_tx_id
        self.next_tx_id += 1
        
        # Update balance
        self.update_balance(client_id, -amount)
        
        return Transaction(
            tx_type=TransactionType.WITHDRAWAL.value,
            client=client_id,
            tx_id=tx_id,
            amount=amount
        )
    
    def generate_dispute(self) -> Optional[Transaction]:
        """Generate a dispute transaction for a random disputable transaction"""
        if not self.disputable_transactions:
            return None
        
        # Pick a random disputable transaction
        tx_id = random.choice(list(self.disputable_transactions.keys()))
        amount = self.disputable_transactions.pop(tx_id)
        self.disputed_transactions[tx_id] = amount
        
        # Find the client for this transaction (need to look it up)
        client_id = None
        for tx in self.transactions:
            if tx.tx_id == tx_id:
                client_id = tx.client
                break
        
        if client_id is None:
            return None
        
        return Transaction(
            tx_type=TransactionType.DISPUTE.value,
            client=client_id,
            tx_id=tx_id,
            amount=None  # Disputes don't include amount
        )
    
    def generate_resolve(self) -> Optional[Transaction]:
        """Generate a resolve transaction for a random disputed transaction"""
        if not self.disputed_transactions:
            return None
        
        # Pick a random disputed transaction
        tx_id = random.choice(list(self.disputed_transactions.keys()))
        amount = self.disputed_transactions.pop(tx_id)
        self.disputable_transactions[tx_id] = amount  # Back to disputable
        
        # Find the client for this transaction
        client_id = None
        for tx in self.transactions:
            if tx.tx_id == tx_id:
                client_id = tx.client
                break
        
        if client_id is None:
            return None
        
        return Transaction(
            tx_type=TransactionType.RESOLVE.value,
            client=client_id,
            tx_id=tx_id,
            amount=None
        )
    
    def generate_chargeback(self) -> Optional[Transaction]:
        """Generate a chargeback transaction for a random disputed transaction"""
        if not self.disputed_transactions:
            return None
        
        # Pick a random disputed transaction
        tx_id = random.choice(list(self.disputed_transactions.keys()))
        amount = self.disputed_transactions.pop(tx_id)  # Remove permanently
        
        # Find the client for this transaction
        client_id = None
        for tx in self.transactions:
            if tx.tx_id == tx_id:
                client_id = tx.client
                break
        
        if client_id is None:
            return None
        
        return Transaction(
            tx_type=TransactionType.CHARGEBACK.value,
            client=client_id,
            tx_id=tx_id,
            amount=None
        )
    
    def generate_transactions(self) -> List[Transaction]:
        """Generate the full set of transactions"""
        # Phase 1: Generate deposits and withdrawals
        base_transactions = int(self.total_transactions * (self.deposit_pct + self.withdrawal_pct))
        deposits_count = int(base_transactions * (self.deposit_pct / (self.deposit_pct + self.withdrawal_pct)))
        withdrawals_count = base_transactions - deposits_count
        
        print(f"Generating {deposits_count} deposits and {withdrawals_count} withdrawals...")
        
        # Generate deposits and withdrawals
        for _ in range(deposits_count):
            client_id = random.randint(1, self.num_clients)
            tx = self.generate_deposit(client_id)
            self.transactions.append(tx)
        
        for _ in range(withdrawals_count):
            client_id = random.randint(1, self.num_clients)
            tx = self.generate_withdrawal(client_id)
            if tx:  # Only add if withdrawal was possible
                self.transactions.append(tx)
        
        # Phase 2: Generate disputes, resolves, and chargebacks
        disputable_count = len(self.disputable_transactions)
        disputes_to_generate = int(disputable_count * self.dispute_pct)
        
        print(f"Generating {disputes_to_generate} disputes from {disputable_count} disputable transactions...")
        
        # Generate disputes
        for _ in range(disputes_to_generate):
            tx = self.generate_dispute()
            if tx:
                self.transactions.append(tx)
        
        # Generate resolves and chargebacks for disputed transactions
        disputed_count = len(self.disputed_transactions)
        resolves_count = int(disputed_count * self.resolve_pct)
        chargebacks_count = int(disputed_count * self.chargeback_pct)
        
        print(f"Generating {resolves_count} resolves and {chargebacks_count} chargebacks...")
        
        # Generate resolves
        for _ in range(resolves_count):
            tx = self.generate_resolve()
            if tx:
                self.transactions.append(tx)
        
        # Generate chargebacks
        for _ in range(chargebacks_count):
            tx = self.generate_chargeback()
            if tx:
                self.transactions.append(tx)
        
        # Shuffle transactions to make them more realistic (not perfectly ordered)
        random.shuffle(self.transactions)
        
        return self.transactions
    
    def write_csv(self, filename: str):
        """Write transactions to CSV file"""
        with open(filename, 'w', newline='') as csvfile:
            writer = csv.writer(csvfile)
            
            # Write header
            writer.writerow(['type', 'client', 'tx', 'amount'])
            
            # Write transactions
            for tx in self.transactions:
                writer.writerow(tx.to_csv_row())
        
        print(f"Generated {len(self.transactions)} transactions in {filename}")
    
    def print_summary(self):
        """Print summary statistics"""
        type_counts = {}
        for tx in self.transactions:
            type_counts[tx.tx_type] = type_counts.get(tx.tx_type, 0) + 1
        
        print("\n=== TRANSACTION SUMMARY ===")
        for tx_type, count in type_counts.items():
            pct = (count / len(self.transactions)) * 100 if self.transactions else 0
            print(f"{tx_type:>10}: {count:>6} ({pct:>5.1f}%)")
        
        print(f"\nTotal transactions: {len(self.transactions)}")
        print(f"Unique clients: {len(self.client_balances)}")
        
        # Print final client balances summary
        balances = list(self.client_balances.values())
        if balances:
            print(f"\nClient balance statistics:")
            print(f"  Total balance: ${sum(balances):,.2f}")
            print(f"  Average: ${sum(balances)/len(balances):,.2f}")
            print(f"  Min: ${min(balances):,.2f}")
            print(f"  Max: ${max(balances):,.2f}")

def main():
    parser = argparse.ArgumentParser(description='Generate test transaction data for payments engine')
    
    parser.add_argument('--clients', type=int, default=1000,
                      help='Number of unique clients (default: 1000)')
    parser.add_argument('--transactions', type=int, default=10000,
                      help='Total number of transactions (default: 10000)')
    parser.add_argument('--output', type=str, default='test_transactions.csv',
                      help='Output CSV filename (default: test_transactions.csv)')
    
    # Transaction type percentages
    parser.add_argument('--deposit-pct', type=float, default=0.6,
                      help='Percentage of deposits (0.0-1.0, default: 0.6)')
    parser.add_argument('--withdrawal-pct', type=float, default=0.3,
                      help='Percentage of withdrawals (0.0-1.0, default: 0.3)')
    
    # Dispute-related percentages
    parser.add_argument('--dispute-pct', type=float, default=0.08,
                      help='Percentage of deposits that get disputed (0.0-1.0, default: 0.08)')
    parser.add_argument('--resolve-pct', type=float, default=0.6,
                      help='Percentage of disputes that get resolved (0.0-1.0, default: 0.6)')
    parser.add_argument('--chargeback-pct', type=float, default=0.3,
                      help='Percentage of disputes that get charged back (0.0-1.0, default: 0.3)')
    
    parser.add_argument('--seed', type=int, default=42,
                      help='Random seed for reproducible results (default: 42)')
    
    args = parser.parse_args()
    
    # Create generator
    generator = TransactionGenerator(
        num_clients=args.clients,
        total_transactions=args.transactions,
        deposit_pct=args.deposit_pct,
        withdrawal_pct=args.withdrawal_pct,
        dispute_pct=args.dispute_pct,
        resolve_pct=args.resolve_pct,
        chargeback_pct=args.chargeback_pct,
        seed=args.seed
    )
    
    # Generate transactions
    transactions = generator.generate_transactions()
    
    # Write to file
    generator.write_csv(args.output)
    
    # Print summary
    generator.print_summary()

if __name__ == '__main__':
    main()
