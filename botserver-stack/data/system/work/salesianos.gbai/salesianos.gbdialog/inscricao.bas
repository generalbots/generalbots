DESCRIPTION "Fazer inscrição para a Escola Salesiana (Confirmar todos os dados com o usuário antes de executar a ferramenta)"
PARAM nomeCompleto AS STRING LIKE "João Silva Santos" DESCRIPTION "Nome completo do aluno"
PARAM dataNascimento AS DATE LIKE "2015-03-15" DESCRIPTION "Data de nascimento (formato ISO: YYYY-MM-DD)"
PARAM nomeResponsavel AS STRING LIKE "Maria Silva Santos" DESCRIPTION "Nome do responsável"
PARAM telefone AS STRING LIKE "(21) 99999-9999" DESCRIPTION "Telefone com DDD"
PARAM email AS EMAIL LIKE "maria.santos@example.com" DESCRIPTION "Email para contato"
PARAM endereco AS STRING LIKE "Rua das Flores, 123 - Centro" DESCRIPTION "Endereço completo"
PARAM serie AS STRING LIKE "8º ano" DESCRIPTION "Série desejada"
PARAM turno AS STRING ENUM ["MANHA", "TARDE"] LIKE "MANHA" DESCRIPTION "Turno: MANHA ou TARDE"


id = "INS-" + FORMAT(NOW(), "yyyyMMdd") + "-" + FORMAT(RANDOM(1000, 9999), "0000")
protocoloNumero = "INS" + FORMAT(RANDOM(100000, 999999), "000000")
dataCadastro = FORMAT(NOW(), "yyyy-MM-dd HH:mm:ss")
status = "AGUARDANDO_ANALISE"

dataNascDisplay = MID(dataNascimento, 9, 2) + "/" + MID(dataNascimento, 6, 2) + "/" + MID(dataNascimento, 1, 4)

turnoDescricao = "Manhã"
IF turno = "TARDE" THEN
    turnoDescricao = "Tarde"
END IF

SAVE "inscricoes", id, protocoloNumero, nomeCompleto, dataNascimento, nomeResponsavel, telefone, email, endereco, serie, turno, dataCadastro, status

SET_BOT_MEMORY("ultima_inscricao", id)
SET_BOT_MEMORY("ultimo_protocolo_inscricao", protocoloNumero)

BEGIN TALK
Inscrição enviada com sucesso!

PROTOCOLO: ${protocoloNumero}
ID: ${id}

Aluno: ${nomeCompleto}
Data de Nascimento: ${dataNascDisplay}
Responsável: ${nomeResponsavel}
Série: ${serie}
Turno: ${turnoDescricao}

Status: Aguardando análise

PRÓXIMOS PASSOS:
   • Você receberá um e-mail com instruções
   • Entraremos em contato em até 3 dias úteis
   • Verifique também sua caixa de spam

Contato da Secretaria:
Telefone: (21) 3333-4444
WhatsApp: (21) 99999-8888
Email: secretaria@salesianos.br

Registro salvo com sucesso e e-mail de confirmação enviado!
END TALK

BEGIN MAIL email
Subject: Confirmação de Inscrição - Protocolo ${protocoloNumero}

Prezado(a) ${nomeResponsavel},

Recebemos com alegria a inscrição de ${nomeCompleto} na Escola Salesiana.

DADOS DA INSCRIÇÃO:
===========================================
Protocolo: ${protocoloNumero}
ID: ${id}
Aluno: ${nomeCompleto}
Data de Nascimento: ${dataNascDisplay}
Responsável: ${nomeResponsavel}
Série: ${serie}
Turno: ${turnoDescricao}
===========================================

PRÓXIMOS PASSOS:
1. Aguarde nosso contato por e-mail ou telefone
2. Providencie a documentação necessária
3. Compareça à secretaria para finalizar a matrícula

Que Deus abençoe sua família!

Atenciosamente,

Secretaria da Escola Salesiana
Tel: (21) 3333-4444 | WhatsApp: (21) 99999-8888
END MAIL

RETURN id
