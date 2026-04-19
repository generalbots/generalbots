PARAM customer_name AS NAME LIKE "João Silva" DESCRIPTION "Customer name for the order"
PARAM items AS OBJECT LIKE "[{id: 1, qty: 2}]" DESCRIPTION "JSON array of items with product id and quantity"

DESCRIPTION "Complete checkout and finalize the sale with customer and cart items"

IF UBOUND(items) = 0 THEN
    TALK "Your cart is empty. Please add items before checkout."
    RETURN NULL
END IF

orderid = "ORD-" + FORMAT(NOW(), "YYYYMMDD") + "-" + FORMAT(RANDOM(1000, 9999))

total = 0
orderitems = []

FOR EACH item IN items
    product = FIND "products.csv", "id = ${item.id}"

    IF product THEN
        subtotal = product.price * item.qty
        total = total + subtotal

        WITH orderitem
            product_id = item.id
            name = product.name
            qty = item.qty
            price = product.price
            subtotal = subtotal
        END WITH

        orderitems[UBOUND(orderitems)] = orderitem
    END IF
NEXT

IF total = 0 THEN
    TALK "No valid products found in cart."
    RETURN NULL
END IF

WITH order
    id = orderid
    customer = customer_name
    totalValue = total
    status = "pending"
    created = NOW()
END WITH

SAVE "orders.csv", order
SAVE "order_items.csv", orderid, TOJSON(orderitems)

SET BOT MEMORY "last_order", orderid

TALK "Order confirmed: " + orderid
TALK "Customer: " + customer_name

FOR EACH orderitem IN orderitems
    TALK "- " + orderitem.name + " x" + orderitem.qty + " = $" + FORMAT(orderitem.subtotal, "#,##0.00")
NEXT

TALK "Total: $" + FORMAT(total, "#,##0.00")

RETURN orderid
