PARAM expression AS STRING LIKE "2 + 2" DESCRIPTION "Mathematical expression to calculate"

DESCRIPTION "Calculate mathematical expressions, conversions, and formulas"

WITH result
    expression = expression
    timestamp = NOW()
END WITH

expr = REPLACE(expression, " ", "")

IF INSTR(expr, "+") > 0 THEN
    parts = SPLIT(expr, "+")
    IF UBOUND(parts) = 2 THEN
        result.answer = VAL(parts[0]) + VAL(parts[1])
        result.operation = "addition"
    END IF

ELSE IF INSTR(expr, "-") > 0 AND LEFT(expr, 1) <> "-" THEN
    parts = SPLIT(expr, "-")
    IF UBOUND(parts) = 2 THEN
        result.answer = VAL(parts[0]) - VAL(parts[1])
        result.operation = "subtraction"
    END IF

ELSE IF INSTR(expr, "*") > 0 THEN
    parts = SPLIT(expr, "*")
    IF UBOUND(parts) = 2 THEN
        result.answer = VAL(parts[0]) * VAL(parts[1])
        result.operation = "multiplication"
    END IF

ELSE IF INSTR(expr, "/") > 0 THEN
    parts = SPLIT(expr, "/")
    IF UBOUND(parts) = 2 THEN
        IF VAL(parts[1]) <> 0 THEN
            result.answer = VAL(parts[0]) / VAL(parts[1])
            result.operation = "division"
        ELSE
            TALK "Error: Division by zero"
            RETURN NULL
        END IF
    END IF

ELSE IF INSTR(LCASE(expr), "sqrt") > 0 THEN
    start_pos = INSTR(LCASE(expr), "sqrt(") + 5
    end_pos = INSTR(start_pos, expr, ")")
    IF end_pos > start_pos THEN
        num = VAL(MID(expr, start_pos, end_pos - start_pos))
        IF num >= 0 THEN
            result.answer = SQR(num)
            result.operation = "square root"
        ELSE
            TALK "Error: Cannot calculate square root of negative number"
            RETURN NULL
        END IF
    END IF

ELSE IF INSTR(expr, "^") > 0 THEN
    parts = SPLIT(expr, "^")
    IF UBOUND(parts) = 2 THEN
        result.answer = VAL(parts[0]) ^ VAL(parts[1])
        result.operation = "power"
    END IF

ELSE IF INSTR(LCASE(expr), "abs") > 0 THEN
    start_pos = INSTR(LCASE(expr), "abs(") + 4
    end_pos = INSTR(start_pos, expr, ")")
    IF end_pos > start_pos THEN
        result.answer = ABS(VAL(MID(expr, start_pos, end_pos - start_pos)))
        result.operation = "absolute value"
    END IF

ELSE IF INSTR(LCASE(expr), "round") > 0 THEN
    start_pos = INSTR(LCASE(expr), "round(") + 6
    end_pos = INSTR(start_pos, expr, ")")
    IF end_pos > start_pos THEN
        result.answer = ROUND(VAL(MID(expr, start_pos, end_pos - start_pos)), 0)
        result.operation = "rounding"
    END IF

ELSE IF INSTR(expr, "%") > 0 AND INSTR(LCASE(expr), "of") > 0 THEN
    expr_lower = LCASE(expr)
    of_pos = INSTR(expr_lower, "of")
    percent_part = REPLACE(LEFT(expr, of_pos - 1), "%", "")
    percent_val = VAL(TRIM(percent_part))
    base_val = VAL(TRIM(MID(expr, of_pos + 2)))
    result.answer = (percent_val / 100) * base_val
    result.operation = "percentage"

ELSE
    result.answer = VAL(expr)
    result.operation = "direct value"
END IF

IF result.answer <> NULL THEN
    TALK "Result: " + result.answer
    RETURN result
ELSE
    TALK "Could not calculate expression"
    RETURN NULL
END IF
