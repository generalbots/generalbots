PARAM image AS STRING

barcode = SCAN BARCODE image
IF NOT barcode THEN
    RETURN WITH error AS "No barcode found"
END IF

ean = barcode.data
cached = FIND "products" WITH "global_trade_number = '" + ean + "'"
IF cached THEN
    RETURN cached
END IF

data = GET "https://world.openfoodfacts.org/api/v0/product/" + ean + ".json"
IF data.product THEN
    SAVE "products" WITH name AS data.product.product_name, brand AS data.product.brands, global_trade_number AS ean, description AS data.product.generic_name, category AS data.product.categories, net_weight AS data.product.quantity
    RETURN FIND "products" WITH "global_trade_number = '" + ean + "'"
END IF

info = LLM "Search product info for EAN: " + ean + ". Return JSON with name, brand, description, category"
SAVE "products" WITH name AS info.name, brand AS info.brand, description AS info.description, category AS info.category, global_trade_number AS ean

RETURN FIND "products" WITH "global_trade_number = '" + ean + "'"
