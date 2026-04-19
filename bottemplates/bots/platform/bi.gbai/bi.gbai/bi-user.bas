' Individual customer report generation

customers = FIND "Customers"

FOR EACH c IN customers
    data = SELECT SUM(UnitPrice * Quantity) AS Value, MONTH(OrderDate)+'/'+YEAR(OrderDate) FROM billing
        JOIN Customers ON billing.CustomerID = Customers.CustomerID
        GROUP BY MONTH(OrderDate), YEAR(OrderDate)
        WHERE Customers.CustomerID = c.CustomerID

    img = CHART "timeseries", data
    SEND FILE img, "Monthly Consumption"
END FOR
