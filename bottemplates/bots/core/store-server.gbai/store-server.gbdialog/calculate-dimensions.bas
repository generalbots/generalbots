PARAM product_id AS STRING
PARAM quantity AS INTEGER

product = PRODUCT product_id
IF NOT product THEN
    RETURN WITH error AS "Product not found"
END IF

IF NOT quantity THEN
    quantity = 1
END IF

IF NOT product.gross_weight THEN
    dims = LLM "Estimate shipping dimensions for: " + product.name + ". Return JSON with length_cm, width_cm, height_cm, weight_kg"
    UPDATE "products" WITH length AS dims.length_cm, width AS dims.width_cm, height AS dims.height_cm, gross_weight AS dims.weight_kg WHERE id = product_id
    product = PRODUCT product_id
END IF

volume = product.length * product.width * product.height * quantity
volumetric_weight = volume / 5000
actual_weight = product.gross_weight * quantity
billable = MAX actual_weight, volumetric_weight


RETURN WITH product_id AS product_id, quantity AS quantity, length AS product.length, width AS product.width, height AS product.height, weight AS product.gross_weight, volume_cm3 AS volume, billable_weight_kg AS billable
