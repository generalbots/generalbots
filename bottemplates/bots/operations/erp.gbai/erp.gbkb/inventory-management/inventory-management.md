# ERP Inventory Management Guide

## Overview

The ERP Inventory Management module provides comprehensive tools for managing stock levels, warehouse operations, purchasing, and inventory tracking across your organization.

## Core Functions

### Receive Inventory

Process incoming goods from purchase orders.

**Steps:**
1. Enter the purchase order number
2. System displays ordered items and quantities
3. Enter actual quantities received for each item
4. System updates stock levels and creates transaction records
5. Notifications sent to relevant personnel

**Key Features:**
- Automatic stock level updates
- Variance tracking (ordered vs. received)
- Cost tracking and average cost calculation
- Receipt transaction logging

### Ship Inventory

Process outgoing shipments for sales orders.

**Steps:**
1. Enter the sales order number
2. System verifies stock availability
3. If sufficient stock, shipment is created
4. Stock levels are deducted
5. Customer receives shipping notification

**Checks Performed:**
- Stock availability verification
- Reserved quantity validation
- Backorder handling

### Check Stock

Query current inventory levels across warehouses.

**Information Displayed:**
- Item name and code
- Quantity on hand per warehouse
- Available quantity (on hand minus reserved)
- Total across all warehouses
- Low stock warnings
- Reorder recommendations

### Transfer Stock

Move inventory between warehouses.

**Process:**
1. Select item to transfer
2. Choose source warehouse
3. Verify available quantity
4. Enter transfer quantity
5. Select destination warehouse
6. System creates transfer records

**Tracking:**
- Transfer numbers for audit trail
- Out/In transaction pairs
- Cost tracking maintained

### Cycle Count

Perform physical inventory counts and adjustments.

**Process:**
1. Select warehouse to count
2. System shows items and system quantities
3. Enter physical counts for each item
4. System calculates variances
5. Automatic adjustments created
6. Report sent to inventory manager

## Data Tables

### Items
- Item code and name
- Category and description
- Unit of measure
- Minimum stock level
- Reorder point and quantity
- Average cost and last cost

### Inventory Stock
- Item reference
- Warehouse location
- Quantity on hand
- Quantity reserved
- Quantity available
- Last movement date
- Last counted date

### Inventory Transactions
- Transaction type (receipt, shipment, transfer, adjustment)
- Transaction number
- Item and warehouse
- Quantity and cost
- Reference (PO, SO, etc.)
- User and timestamp

### Warehouses
- Warehouse code and name
- Location address
- Contact information
- Capacity limits

## Alerts and Notifications

### Low Stock Alerts
Triggered when available quantity falls below minimum stock level.

### Reorder Notifications
Automatic task creation when stock reaches reorder point.

### Receipt Confirmations
Email sent to buyer when purchase order is received.

### Shipment Notifications
Customer notified when order ships with tracking information.

## Best Practices

### Daily Operations
1. Process all receipts promptly
2. Ship orders in FIFO sequence
3. Review low stock alerts
4. Address reorder recommendations

### Weekly Tasks
1. Review inventory transaction reports
2. Investigate any discrepancies
3. Plan upcoming transfers
4. Update reorder points as needed

### Monthly Tasks
1. Conduct cycle counts by zone
2. Review slow-moving inventory
3. Analyze inventory turnover
4. Adjust minimum stock levels

### Year-End
1. Complete full physical inventory
2. Reconcile all variances
3. Review and adjust costs
4. Archive transaction history

## Frequently Asked Questions

**Q: How is average cost calculated?**
A: Average cost is recalculated with each receipt: (existing value + new receipt value) / total quantity.

**Q: Can I transfer reserved inventory?**
A: No, only available (unreserved) inventory can be transferred.

**Q: What happens if I receive more than ordered?**
A: The system accepts over-receipts and updates the PO line accordingly.

**Q: How do I handle damaged goods?**
A: Use cycle count with an adjustment to remove damaged items, noting the reason.

**Q: Can I undo a shipment?**
A: Contact your administrator to process a return receipt transaction.

**Q: How do I set up a new warehouse?**
A: Add the warehouse to the warehouses table with code, name, and location details.

## Troubleshooting

### Purchase Order Not Found
- Verify the PO number is correct
- Check if PO status is "open" or "partial"
- Ensure you have access to the vendor

### Insufficient Stock for Shipment
- Check available quantity vs. ordered
- Review reserved quantities
- Consider warehouse transfers

### Cycle Count Variance
- Verify physical count accuracy
- Check for unprocessed receipts or shipments
- Review recent transfers

### Transfer Failures
- Verify source warehouse has stock
- Check available (not reserved) quantity
- Ensure destination warehouse is active

## Reports

Available inventory reports:
- **Stock Status**: Current levels by item/warehouse
- **Transaction History**: All movements for date range
- **Aging Report**: Stock age by receipt date
- **Valuation Report**: Inventory value by category
- **Turnover Report**: Movement frequency analysis