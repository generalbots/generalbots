PARAM product AS STRING LIKE "fax" DESCRIPTION "Name of the product to get price for"

DESCRIPTION "Get the price of a product by name from the product catalog"

productRecord = FIND "products.csv", "name = ${product}"

IF productRecord THEN
    RETURN productRecord.price
ELSE
    RETURN -1
END IF
