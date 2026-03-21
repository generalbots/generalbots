TABLE inscricoes
    FIELD id AS STRING
    FIELD protocoloNumero AS STRING
    FIELD nomeCompleto AS STRING
    FIELD dataNascimento AS DATE
    FIELD nomeResponsavel AS STRING
    FIELD telefone AS STRING
    FIELD email AS STRING
    FIELD endereco AS STRING
    FIELD serie AS STRING
    FIELD turno AS STRING
    FIELD dataCadastro AS DATETIME
    FIELD status AS STRING
END TABLE

TABLE agendamentos_visita
    FIELD id AS STRING
    FIELD protocoloNumero AS STRING
    FIELD nomeResponsavel AS STRING
    FIELD telefone AS STRING
    FIELD email AS STRING
    FIELD dataVisita AS DATE
    FIELD horario AS STRING
    FIELD numeroVisitantes AS INTEGER
    FIELD dataCadastro AS DATETIME
    FIELD status AS STRING
END TABLE

TABLE atendimentos
    FIELD id AS STRING
    FIELD protocoloNumero AS STRING
    FIELD nome AS STRING
    FIELD assunto AS STRING
    FIELD mensagem AS STRING
    FIELD telefone AS STRING
    FIELD email AS STRING
    FIELD dataCadastro AS DATETIME
    FIELD status AS STRING
END TABLE
