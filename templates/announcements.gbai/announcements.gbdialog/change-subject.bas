PARAM subject as string
DESCRIPTION "Chamado quando alguém quer mudar o assunto da conversa."

kbname = LLM "Devolva uma única palavra circular, comunicado ou geral de acordo com a seguinte frase:" + subject

ADD_KB kbname


TALK "You have chosen to change the subject to " + subject + "."