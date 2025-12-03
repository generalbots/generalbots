# Store Template

The store template provides a complete e-commerce assistant that helps customers browse products, manage shopping carts, and complete purchases through conversational AI.

## Topic: E-Commerce & Shopping Assistant

This template is perfect for:
- Online retail stores
- Product catalog browsing
- Shopping cart management
- Order tracking
- Customer support for e-commerce

## The Code

```basic
ADD TOOL "checkout"
ADD TOOL "search-product"
ADD TOOL "add-to-cart"
ADD TOOL "view-cart"
ADD TOOL "track-order"
ADD TOOL "product-details"

data = FIND "products.csv"

CLEAR SUGGESTIONS

ADD SUGGESTION "products" AS "View products"
ADD SUGGESTION "cart" AS "View my cart"
ADD SUGGESTION "checkout" AS "Checkout"
ADD SUGGESTION "orders" AS "Track my order"
ADD SUGGESTION "help" AS "Shopping help"

SET CONTEXT "store" AS "You are a virtual store sales assistant. Help customers browse products, add items to cart, and complete purchases. Be friendly and helpful. Available products: ${TOJSON(data)}"

BEGIN TALK
**Virtual Store**

Welcome! I can help you with:
• Browse our product catalog
• Add items to your cart
• Complete your purchase
• Track your orders

Select an option or tell me what you're looking for.
END TALK

BEGIN SYSTEM PROMPT
You are a friendly sales assistant in our virtual store.

Welcome customers warmly.
Help them find products.
Provide clear product information.
Guide through purchase process.
Offer assistance when needed.

Product catalog is available in context.
Suggest related products when appropriate.
Confirm items before adding to cart.
END SYSTEM PROMPT
```

## Sample Dialogs

These conversations show how the store template works in real-world scenarios.

### Dialog 1: Product Search

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🛒</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Virtual Store</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Virtual Store</div>
      <p><strong>Virtual Store</strong></p>
      <p>Welcome! I can help you with:</p>
      <p>• Browse our product catalog<br>• Add items to your cart<br>• Complete your purchase<br>• Track your orders</p>
      <div class="wa-time">10:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I'm looking for wireless headphones</p>
      <div class="wa-time">10:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🎧 Great choice! Here are our wireless headphones:</p>
      <p></p>
      <p><strong>1. ProSound Elite</strong></p>
      <p>💰 $149.99 | ⭐ 4.8/5</p>
      <p>Active noise canceling, 30hr battery</p>
      <p></p>
      <p><strong>2. BassBuds Pro</strong></p>
      <p>💰 $89.99 | ⭐ 4.5/5</p>
      <p>Deep bass, 20hr battery</p>
      <p></p>
      <p><strong>3. SportFit Wireless</strong></p>
      <p>💰 $69.99 | ⭐ 4.6/5</p>
      <p>Sweat-proof, secure fit</p>
      <p></p>
      <p>Would you like more details on any of these?</p>
      <div class="wa-time">10:01</div>
    </div>
  </div>
</div>

### Dialog 2: Add to Cart and Checkout

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🛒</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Virtual Store</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Add the ProSound Elite to my cart</p>
      <div class="wa-time">10:05 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Added to cart!</p>
      <p><strong>ProSound Elite</strong> - $149.99</p>
      <p></p>
      <p>🛒 Your cart: 1 item ($149.99)</p>
      <p></p>
      <p>Would you like to:</p>
      <p>• Continue shopping</p>
      <p>• View cart</p>
      <p>• Proceed to checkout</p>
      <div class="wa-time">10:05</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Checkout</p>
      <div class="wa-time">10:06 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📦 <strong>Checkout</strong></p>
      <p></p>
      <p><strong>Order Summary:</strong></p>
      <p>1x ProSound Elite - $149.99</p>
      <p>Shipping - $9.99</p>
      <p><strong>Total: $159.98</strong></p>
      <p></p>
      <p>Shipping to: 123 Main St, Apt 4B</p>
      <p></p>
      <p>Please select payment method:</p>
      <p>1️⃣ Credit Card (****4521)</p>
      <p>2️⃣ PayPal</p>
      <p>3️⃣ PIX</p>
      <div class="wa-time">10:06</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>1</p>
      <div class="wa-time">10:07 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ <strong>Order Confirmed!</strong></p>
      <p></p>
      <p>Order #ORD-2025-0314-001</p>
      <p>Total: $159.98</p>
      <p></p>
      <p>📧 Confirmation sent to your email</p>
      <p>🚚 Estimated delivery: March 18-20</p>
      <p></p>
      <p>Thank you for shopping with us! 🎉</p>
      <div class="wa-time">10:07</div>
    </div>
  </div>
</div>

### Dialog 3: Order Tracking

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🛒</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Virtual Store</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Where is my order?</p>
      <div class="wa-time">14:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📦 <strong>Order Status</strong></p>
      <p></p>
      <p>Order #ORD-2025-0314-001</p>
      <p></p>
      <p>✅ Order placed - Mar 14</p>
      <p>✅ Payment confirmed - Mar 14</p>
      <p>✅ Shipped - Mar 15</p>
      <p>🚚 <strong>Out for delivery</strong> - Mar 17</p>
      <p>⏳ Delivered - Expected today</p>
      <p></p>
      <p>Tracking: <a href="#">ABC123456789</a></p>
      <p></p>
      <p>Your package is on the delivery truck and should arrive by 6 PM today!</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>
</div>

### Dialog 4: Product Recommendations

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🛒</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Virtual Store</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I need a gift for my brother who loves gaming</p>
      <div class="wa-time">16:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🎮 Great! Here are some popular gaming gifts:</p>
      <p></p>
      <p><strong>Under $50:</strong></p>
      <p>• Gaming Mouse Pad XL - $29.99</p>
      <p>• LED Light Strip for Setup - $24.99</p>
      <p></p>
      <p><strong>$50-$100:</strong></p>
      <p>• Mechanical Keyboard RGB - $79.99</p>
      <p>• Gaming Headset Pro - $69.99</p>
      <p></p>
      <p><strong>$100+:</strong></p>
      <p>• Gaming Mouse Wireless - $129.99</p>
      <p>• Streaming Microphone Kit - $149.99</p>
      <p></p>
      <p>What's your budget? I can help you find the perfect gift! 🎁</p>
      <div class="wa-time">16:00</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `ADD TOOL` | Register e-commerce tools |
| `FIND` | Load product catalog from CSV |
| `ADD SUGGESTION` | Create quick action buttons |
| `SET CONTEXT` | Define store context with product data |
| `BEGIN TALK` | Welcome message block |
| `BEGIN SYSTEM PROMPT` | Sales assistant behavior rules |

## Template Structure

```
store.gbai/
├── store.gbdialog/
│   ├── start.bas           # Main entry point
│   └── checkout.bas        # Checkout process
├── store.gbdata/
│   └── products.csv        # Product catalog
└── store.gbot/
    └── config.csv          # Bot configuration
```

## Checkout Tool: checkout.bas

```basic
PARAM confirm AS STRING LIKE "yes" DESCRIPTION "Confirm order placement"

DESCRIPTION "Complete the purchase and process payment"

' Get cart from memory
cart = GET BOT MEMORY("cart_" + user_id)

IF UBOUND(cart) = 0 THEN
    TALK "Your cart is empty. Add some items first!"
    RETURN NULL
END IF

' Calculate totals
subtotal = 0
FOR EACH item IN cart
    subtotal = subtotal + (item.price * item.quantity)
NEXT

shipping = 9.99
IF subtotal > 100 THEN
    shipping = 0  ' Free shipping over $100
END IF

total = subtotal + shipping

' Show order summary
TALK "📦 **Order Summary**"
TALK ""
FOR EACH item IN cart
    TALK item.quantity + "x " + item.name + " - $" + FORMAT(item.price * item.quantity, "#,##0.00")
NEXT
TALK ""
TALK "Subtotal: $" + FORMAT(subtotal, "#,##0.00")
IF shipping = 0 THEN
    TALK "Shipping: FREE ✨"
ELSE
    TALK "Shipping: $" + FORMAT(shipping, "#,##0.00")
END IF
TALK "**Total: $" + FORMAT(total, "#,##0.00") + "**"
TALK ""
TALK "Type CONFIRM to place your order."

HEAR confirmation

IF UPPER(confirmation) = "CONFIRM" THEN
    ' Create order
    orderNumber = "ORD-" + FORMAT(NOW(), "YYYY-MMDD") + "-" + FORMAT(RANDOM(100, 999))
    
    WITH order
        id = orderNumber
        user_id = user_id
        items = TOJSON(cart)
        subtotal = subtotal
        shipping = shipping
        total = total
        status = "confirmed"
        created_at = NOW()
    END WITH
    
    SAVE "orders.csv", order
    
    ' Clear cart
    SET BOT MEMORY "cart_" + user_id, []
    
    ' Send confirmation email
    SEND MAIL user_email, "Order Confirmed - " + orderNumber, 
        "Thank you for your order!\n\nOrder: " + orderNumber + "\nTotal: $" + total
    
    TALK "✅ **Order Confirmed!**"
    TALK "Order #" + orderNumber
    TALK "📧 Confirmation sent to your email"
    TALK "🚚 Estimated delivery: 3-5 business days"
    TALK ""
    TALK "Thank you for shopping with us! 🎉"
    
    RETURN orderNumber
ELSE
    TALK "Order cancelled. Your cart items are saved."
    RETURN NULL
END IF
```

## Add to Cart Tool: add-to-cart.bas

```basic
PARAM product_id AS STRING LIKE "PROD001" DESCRIPTION "Product ID to add"
PARAM quantity AS INTEGER LIKE 1 DESCRIPTION "Quantity to add"

DESCRIPTION "Add a product to the shopping cart"

IF NOT quantity THEN
    quantity = 1
END IF

' Find product
product = FIND "products.csv", "id = '" + product_id + "'"

IF NOT product THEN
    TALK "Sorry, I couldn't find that product. Please try again."
    RETURN NULL
END IF

' Get current cart
cart = GET BOT MEMORY("cart_" + user_id)
IF NOT cart THEN
    cart = []
END IF

' Check if product already in cart
found = FALSE
FOR i = 1 TO UBOUND(cart)
    IF cart[i].product_id = product_id THEN
        cart[i].quantity = cart[i].quantity + quantity
        found = TRUE
        EXIT FOR
    END IF
NEXT

' Add new item if not found
IF NOT found THEN
    WITH item
        product_id = product_id
        name = product.name
        price = product.price
        quantity = quantity
    END WITH
    
    cart = APPEND(cart, item)
END IF

' Save cart
SET BOT MEMORY "cart_" + user_id, cart

' Calculate cart total
cartTotal = 0
cartCount = 0
FOR EACH item IN cart
    cartTotal = cartTotal + (item.price * item.quantity)
    cartCount = cartCount + item.quantity
NEXT

TALK "✅ Added to cart!"
TALK "**" + product.name + "** - $" + FORMAT(product.price, "#,##0.00")
TALK ""
TALK "🛒 Your cart: " + cartCount + " items ($" + FORMAT(cartTotal, "#,##0.00") + ")"

' Suggest related products
IF product.category THEN
    related = FIND "products.csv", "category = '" + product.category + "' AND id <> '" + product_id + "'"
    IF UBOUND(related) > 0 THEN
        TALK ""
        TALK "You might also like: **" + related[1].name + "** - $" + FORMAT(related[1].price, "#,##0.00")
    END IF
END IF

RETURN cart
```

## Customization Ideas

### Add Product Reviews

```basic
ADD TOOL "show-reviews"

' In show-reviews.bas
PARAM product_id AS STRING DESCRIPTION "Product to show reviews for"

reviews = FIND "reviews.csv", "product_id = '" + product_id + "'"

IF UBOUND(reviews) = 0 THEN
    TALK "No reviews yet for this product."
    RETURN
END IF

avgRating = 0
FOR EACH review IN reviews
    avgRating = avgRating + review.rating
NEXT
avgRating = avgRating / UBOUND(reviews)

TALK "⭐ **Customer Reviews** (" + FORMAT(avgRating, "#.#") + "/5)"
TALK ""

FOR EACH review IN FIRST(reviews, 3)
    TALK "**" + review.author + "** - " + STRING(review.rating, "⭐")
    TALK review.comment
    TALK ""
NEXT
```

### Add Discount Codes

```basic
PARAM code AS STRING DESCRIPTION "Discount code to apply"

discount = FIND "discounts.csv", "code = '" + UPPER(code) + "' AND valid_until >= '" + FORMAT(NOW(), "YYYY-MM-DD") + "'"

IF NOT discount THEN
    TALK "Sorry, that code is invalid or expired."
    RETURN NULL
END IF

SET BOT MEMORY "discount_" + user_id, discount

TALK "✅ Discount applied!"
TALK "**" + discount.description + "**"
IF discount.type = "percent" THEN
    TALK "You'll save " + discount.value + "% on your order!"
ELSE
    TALK "You'll save $" + FORMAT(discount.value, "#,##0.00") + " on your order!"
END IF
```

### Add Wishlist Feature

```basic
ADD TOOL "add-to-wishlist"
ADD TOOL "view-wishlist"

' In add-to-wishlist.bas
PARAM product_id AS STRING DESCRIPTION "Product to add to wishlist"

wishlist = GET USER MEMORY("wishlist")
IF NOT wishlist THEN
    wishlist = []
END IF

wishlist = APPEND(wishlist, product_id)
SET USER MEMORY "wishlist", wishlist

product = FIND "products.csv", "id = '" + product_id + "'"
TALK "❤️ Added **" + product.name + "** to your wishlist!"
```

### Add Inventory Check

```basic
' Before adding to cart, check stock
stock = FIND "inventory.csv", "product_id = '" + product_id + "'"

IF stock.quantity < quantity THEN
    IF stock.quantity = 0 THEN
        TALK "😔 Sorry, this item is out of stock."
        TALK "Would you like to be notified when it's available?"
    ELSE
        TALK "⚠️ Only " + stock.quantity + " left in stock."
        TALK "Would you like to add " + stock.quantity + " instead?"
    END IF
    RETURN NULL
END IF
```

## Related Templates

- [bank.bas](./bank.md) - Payment processing integration
- [broadcast.bas](./broadcast.md) - Marketing campaigns
- [talk-to-data.bas](./talk-to-data.md) - Sales analytics

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:600px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-chat::after{content:'';display:table;clear:both}
.wa-message{clear:both;margin-bottom:10px;max-width:85%;position:relative}
.wa-message.user{float:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;border-radius:8px 0 8px 8px;margin-left:40px}
.wa-message.bot{float:left}
.wa-message.bot .wa-bubble{background-color:#fff;border-radius:0 8px 8px 8px;margin-right:40px}
.wa-bubble{padding:8px 12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
.wa-message.user .wa-time{color:#61a05e}
.wa-sender{font-size:12px;font-weight:600;color:#06cf9c;margin-bottom:2px}
.wa-status.read::after{content:'✓✓';color:#53bdeb;margin-left:4px}
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>