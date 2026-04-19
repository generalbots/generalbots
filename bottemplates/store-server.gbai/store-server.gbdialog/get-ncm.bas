PARAM product_id AS STRING

product = PRODUCT product_id
IF NOT product THEN
    RETURN WITH error AS "Product not found"
END IF

IF product.tax_code THEN
    RETURN WITH tax_code AS product.tax_code, tax_class AS product.tax_class
END IF

info = LLM "Get NCM/tax classification for: " + product.name + " " + product.category + ". Return JSON with tax_code, tax_class, description"
UPDATE "products" WITH tax_code AS info.tax_code, tax_class AS info.tax_class WHERE id = product_id

RETURN WITH tax_code AS info.tax_code, tax_class AS info.tax_class, description AS info.description
