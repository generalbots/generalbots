DESCRIPTION "Listar documentos necessários para matrícula, transferência ou renovação na Escola Salesiana"
PARAM tipo AS STRING ENUM ["NOVA_MATRICULA", "TRANSFERENCIA", "RENOVACAO"] LIKE "NOVA_MATRICULA" DESCRIPTION "Tipo: NOVA_MATRICULA para nova matrícula, TRANSFERENCIA para transferência, RENOVACAO para renovação"


resultado = ""

IF tipo = "NOVA_MATRICULA" THEN
    resultado = "Documentos para Nova Matrícula - DOCUMENTOS DO ALUNO: Certidão de nascimento (original e cópia), CPF (se possuir), Histórico escolar original, Declaração de transferência (se vier de outra escola), Comprovante de residência (últimos 3 meses), 2 fotos 3x4 recentes, Carteira de vacinação atualizada. DOCUMENTOS DOS RESPONSÁVEIS: RG e CPF (pai e/ou mãe), Comprovante de renda (para análise de bolsa). Prazo para entrega: até 30 dias antes do início das aulas."
END IF

IF tipo = "TRANSFERENCIA" THEN
    resultado = "Documentos para Transferência: Histórico escolar original, Declaração de transferência da escola de origem, Certidão de nascimento (cópia autenticada), Comprovante de residência (últimos 3 meses), 2 fotos 3x4 recentes, Boletim escolar do ano atual. Documentos estrangeiros precisam de tradução juramentada."
END IF

IF tipo = "RENOVACAO" THEN
    resultado = "Documentos para Renovação de Matrícula: Comprovante de residência atualizado, Requerimento de renovação (fornecido pela escola), Comprovantes de pagamento em dia. Cópias podem ser feitas na secretaria."
END IF

TALK resultado
RETURN resultado
