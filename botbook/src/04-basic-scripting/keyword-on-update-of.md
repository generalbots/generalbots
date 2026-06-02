# ON UPDATE OF

Monitora uma tabela do banco de dados e executa um script automaticamente sempre que um registro é alterado (INSERT, UPDATE ou DELETE).

## Sintaxe

```basic
ON UPDATE OF "nome_da_tabela"
```

O script deve ser nomeado como `{tabela}_update.bas` e colocado na pasta `.gbdialog/` do bot.

## Parâmetros

| Parâmetro | Tipo | Descrição |
|-----------|------|-----------|
| `nome_da_tabela` | String | Nome da tabela a ser monitorada (definida em `tables.bas`) |

## Descrição

O `ON UPDATE OF` é uma declaração especial que deve ser a **primeira linha** do arquivo `.bas`. Ele registra o script como um **gatilho (trigger)** que o DriveMonitor do BotServer observa continuamente. Quando qualquer operação de escrita (INSERT, UPDATE ou DELETE) ocorre na tabela especificada, o script é executado automaticamente.

### Ciclo de Detecção

1. O DriveMonitor observa o bucket MinIO do bot a cada ~10 segundos
2. Quando encontra um arquivo `{tabela}_update.bas`, registra o gatilho
3. O serviço `sync_bot_tables()` monitora o banco de dados PostgreSQL
4. Ao detectar uma alteração, o BotServer executa o script automaticamente
5. O script tem acesso às variáveis `TRIGGER_*` com o contexto da operação
6. O resultado (notificações, logs, etc.) é processado em tempo real

## Variáveis de Gatilho (TRIGGER_*)

Dentro do script executado por `ON UPDATE OF`, as seguintes variáveis estão disponíveis:

| Variável | Tipo | Descrição |
|----------|------|-----------|
| `TRIGGER_ROW_ID` | UUID | Identificador único do registro alterado |
| `TRIGGER_OPERATION` | String | Tipo de operação: `INSERT`, `UPDATE` ou `DELETE` |
| `TRIGGER_TABLE` | String | Nome da tabela onde ocorreu a alteração |
| `TRIGGER_TIMESTAMP` | DateTime | Momento exato em que a alteração foi detectada |
| `TRIGGER_USER` | String | Identificador do usuário que realizou a operação (se disponível) |
| `TRIGGER_OLD_VALUES` | Objeto | Valores anteriores do registro (apenas em UPDATE e DELETE) |
| `TRIGGER_NEW_VALUES` | Objeto | Novos valores do registro (apenas em INSERT e UPDATE) |

## Exemplos

### Notificação de Atualização de Batizado

Arquivo: `batizados_update.bas`

```basic
ON UPDATE OF "batizados"

admin_email = GET BOT MEMORY "admin_email"
whatsapp_manager = GET BOT MEMORY "whatsapp_manager"
registro = GET FROM batizados WHERE id = TRIGGER_ROW_ID

mensagem = "🕊️ Batizado atualizado!\n"
mensagem = mensagem + "Nome: " + registro.nome + "\n"
mensagem = mensagem + "Status: " + TRIGGER_OPERATION + "\n"
mensagem = mensagem + "Data: " + registro.data_batizado

SEND WHATSAPP whatsapp_manager, mensagem
SEND MAIL admin_email, "Batizado Atualizado", mensagem

SAVE {
    "tabela": "batizados",
    "registro_id": TRIGGER_ROW_ID,
    "operacao": TRIGGER_OPERATION,
    "mensagem": mensagem,
    "data": NOW()
} TO historico_notificacoes
```

### Notificação de Doação

Arquivo: `doacoes_update.bas`

```basic
ON UPDATE OF "doacoes"

admin_email = GET BOT MEMORY "admin_email"
whatsapp_manager = GET BOT MEMORY "whatsapp_manager"
registro = GET FROM doacoes WHERE id = TRIGGER_ROW_ID

mensagem = "🎁 Doação atualizada!\n"
mensagem = mensagem + "Doador: " + registro.doador + "\n"
mensagem = mensagem + "Status: " + TRIGGER_OPERATION + "\n"
mensagem = mensagem + "Valor: " + registro.valor

SEND WHATSAPP whatsapp_manager, mensagem
SEND MAIL admin_email, "Doação Atualizada", mensagem

SAVE {
    "tabela": "doacoes",
    "registro_id": TRIGGER_ROW_ID,
    "operacao": TRIGGER_OPERATION,
    "mensagem": mensagem,
    "data": NOW()
} TO historico_notificacoes
```

## Casos de Uso Típicos

| Cenário | Descrição |
|---------|-----------|
| Notificação administrativa | Alertar gestores quando um registro é alterado |
| Sincronização externa | Atualizar sistemas externos via webhook |
| Auditoria | Registrar histórico completo de alterações |
| Fluxo de aprovação | Disparar notificações quando um status muda |
| Atualização de cache | Invalidar cache quando dados são modificados |
| Integração contábil | Enviar alterações para sistema financeiro |

## Boas Práticas

1. **Nomeie corretamente**: O arquivo DEVE seguir o padrão `{tabela}_update.bas` — o nome da tabela em `tables.bas` seguido de `_update`
2. **Seja idempotente**: O mesmo gatilho pode disparar múltiplas vezes para a mesma operação; use verificações como `IF registro.status = "CONCLUIDO" THEN` para evitar duplicidade
3. **Use TRIGGER_ROW_ID**: Sempre busque o registro completo com `GET FROM tabela WHERE id = TRIGGER_ROW_ID` para ter acesso a todos os campos
4. **Mantenha leve**: Scripts de gatilho devem ser rápidos — evite operações pesadas como chamadas LLM ou loops extensos
5. **Não modifique a mesma tabela**: Evite SAVE/UPDATE na mesma tabela que disparou o gatilho para não criar loops infinitos

## Limitações

- Um único arquivo `_update.bas` monitora apenas uma tabela
- O nome do arquivo deve corresponder exatamente ao nome da tabela em `tables.bas`
- Alterações no arquivo requerem ~10s para serem detectadas pelo DriveMonitor
- A tabela monitorada deve existir e estar definida em `tables.bas`

## Palavras-chave Relacionadas

- [ON](./keyword-on.md) — Gerenciador de eventos em geral
- [GET](./keyword-get.md) — Consultar registros no banco
- [SAVE](./keyword-save.md) — Persistir dados
- [GET BOT MEMORY](./keyword-get-bot-memory.md) — Recuperar dados persistentes do bot
- [SEND MAIL](./keyword-send-mail.md) — Enviar notificações por email

## Implementação

Localizado em `src/basic/keywords/on_update_of.rs`

A implementação:
- Parseia a declaração `ON UPDATE OF "tabela"` como primeira linha do arquivo
- Registra o arquivo como gatilho no `DriveMonitor`
- O monitor observa mudanças no PostgreSQL via `sync_bot_tables()`
- Quando uma alteração é detectada, executa o script em um contexto isolado
- Injeta as variáveis `TRIGGER_*` no ambiente de execução
- Garante que o script não seja executado concorrentemente para o mesmo registro
