REM General Bots: CALCULATE Keyword - Universal Math Calculator
REM Perform mathematical calculations and conversions
REM Can be used by ANY template that needs math operations

PARAM expression AS string LIKE "2 + 2"
DESCRIPTION "Calculate mathematical expressions, conversions, and formulas"

REM Validate input
IF NOT expression OR expression = "" THEN
    TALK "❌ Please provide a mathematical expression"
    TALK "💡 Examples: '2 + 2', '10 * 5', '100 / 4', 'sqrt(16)', 'sin(45)'"
    RETURN NULL
END IF

TALK "🧮 Calculating: " + expression

REM Create result object
result = NEW OBJECT
result.expression = expression
result.timestamp = NOW()

REM Try to evaluate the expression
REM This is a simplified calculator - extend as needed

REM Remove spaces
expr = REPLACE(expression, " ", "")

REM Basic operations
IF INSTR(expr, "+") > 0 THEN
    parts = SPLIT(expr, "+")
    IF UBOUND(parts) = 2 THEN
        num1 = VAL(parts[0])
        num2 = VAL(parts[1])
        answer = num1 + num2
        result.answer = answer
        result.operation = "addition"
    END IF

ELSE IF INSTR(expr, "-") > 0 AND LEFT(expr, 1) <> "-" THEN
    parts = SPLIT(expr, "-")
    IF UBOUND(parts) = 2 THEN
        num1 = VAL(parts[0])
        num2 = VAL(parts[1])
        answer = num1 - num2
        result.answer = answer
        result.operation = "subtraction"
    END IF

ELSE IF INSTR(expr, "*") > 0 THEN
    parts = SPLIT(expr, "*")
    IF UBOUND(parts) = 2 THEN
        num1 = VAL(parts[0])
        num2 = VAL(parts[1])
        answer = num1 * num2
        result.answer = answer
        result.operation = "multiplication"
    END IF

ELSE IF INSTR(expr, "/") > 0 THEN
    parts = SPLIT(expr, "/")
    IF UBOUND(parts) = 2 THEN
        num1 = VAL(parts[0])
        num2 = VAL(parts[1])
        IF num2 <> 0 THEN
            answer = num1 / num2
            result.answer = answer
            result.operation = "division"
        ELSE
            TALK "❌ Error: Division by zero"
            RETURN NULL
        END IF
    END IF

ELSE IF INSTR(LCASE(expr), "sqrt") > 0 THEN
    REM Square root
    start_pos = INSTR(LCASE(expr), "sqrt(") + 5
    end_pos = INSTR(start_pos, expr, ")")
    IF end_pos > start_pos THEN
        num_str = MID(expr, start_pos, end_pos - start_pos)
        num = VAL(num_str)
        IF num >= 0 THEN
            answer = SQR(num)
            result.answer = answer
            result.operation = "square root"
        ELSE
            TALK "❌ Error: Cannot calculate square root of negative number"
            RETURN NULL
        END IF
    END IF

ELSE IF INSTR(LCASE(expr), "pow") > 0 OR INSTR(expr, "^") > 0 THEN
    REM Power operation
    IF INSTR(expr, "^") > 0 THEN
        parts = SPLIT(expr, "^")
        IF UBOUND(parts) = 2 THEN
            base = VAL(parts[0])
            exponent = VAL(parts[1])
            answer = base ^ exponent
            result.answer = answer
            result.operation = "power"
        END IF
    END IF

ELSE IF INSTR(LCASE(expr), "abs") > 0 THEN
    REM Absolute value
    start_pos = INSTR(LCASE(expr), "abs(") + 4
    end_pos = INSTR(start_pos, expr, ")")
    IF end_pos > start_pos THEN
        num_str = MID(expr, start_pos, end_pos - start_pos)
        num = VAL(num_str)
        answer = ABS(num)
        result.answer = answer
        result.operation = "absolute value"
    END IF

ELSE IF INSTR(LCASE(expr), "round") > 0 THEN
    REM Rounding
    start_pos = INSTR(LCASE(expr), "round(") + 6
    end_pos = INSTR(start_pos, expr, ")")
    IF end_pos > start_pos THEN
        num_str = MID(expr, start_pos, end_pos - start_pos)
        num = VAL(num_str)
        answer = ROUND(num, 0)
        result.answer = answer
        result.operation = "rounding"
    END IF

ELSE IF INSTR(LCASE(expr), "ceil") > 0 THEN
    REM Ceiling
    start_pos = INSTR(LCASE(expr), "ceil(") + 5
    end_pos = INSTR(start_pos, expr, ")")
    IF end_pos > start_pos THEN
        num_str = MID(expr, start_pos, end_pos - start_pos)
        num = VAL(num_str)
        answer = INT(num)
        IF num > answer THEN
            answer = answer + 1
        END IF
        result.answer = answer
        result.operation = "ceiling"
    END IF

ELSE IF INSTR(LCASE(expr), "floor") > 0 THEN
    REM Floor
    start_pos = INSTR(LCASE(expr), "floor(") + 6
    end_pos = INSTR(start_pos, expr, ")")
    IF end_pos > start_pos THEN
        num_str = MID(expr, start_pos, end_pos - start_pos)
        num = VAL(num_str)
        answer = INT(num)
        result.answer = answer
        result.operation = "floor"
    END IF

ELSE IF INSTR(LCASE(expr), "percent") > 0 OR INSTR(expr, "%") > 0 THEN
    REM Percentage calculation
    REM Format: "20% of 100" or "20 percent of 100"
    expr_lower = LCASE(expr)

    IF INSTR(expr_lower, "of") > 0 THEN
        REM Extract percentage and base number
        of_pos = INSTR(expr_lower, "of")
        percent_part = LEFT(expr, of_pos - 1)
        percent_part = REPLACE(percent_part, "%", "")
        percent_part = REPLACE(LCASE(percent_part), "percent", "")
        percent_val = VAL(TRIM(percent_part))

        base_part = MID(expr, of_pos + 2)
        base_val = VAL(TRIM(base_part))

        answer = (percent_val / 100) * base_val
        result.answer = answer
        result.operation = "percentage"
        result.details = percent_val + "% of " + base_val + " = " + answer
    END IF

ELSE
    REM Try direct evaluation (single number)
    answer = VAL(expr)
    result.answer = answer
    result.operation = "direct value"
END IF

REM Display result
IF result.answer <> NULL THEN
    TALK "✅ Result: " + result.answer
    TALK ""
    TALK "📊 Details:"
    TALK "Expression: " + expression
    TALK "Operation: " + result.operation
    TALK "Answer: " + result.answer

    IF result.details THEN
        TALK result.details
    END IF

    RETURN result
ELSE
    TALK "❌ Could not calculate expression"
    TALK ""
    TALK "💡 Supported operations:"
    TALK "• Basic: + - * /"
    TALK "• Power: 2^3 or pow(2,3)"
    TALK "• Square root: sqrt(16)"
    TALK "• Absolute: abs(-5)"
    TALK "• Rounding: round(3.7), ceil(3.2), floor(3.9)"
    TALK "• Percentage: 20% of 100"
    TALK ""
    TALK "Examples:"
    TALK "• 15 + 25"
    TALK "• 100 / 4"
    TALK "• sqrt(144)"
    TALK "• 2^10"
    TALK "• 15% of 200"

    RETURN NULL
END IF
