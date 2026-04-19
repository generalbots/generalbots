PARAM topic AS STRING LIKE "Novo produto de skincare" DESCRIPTION "Tema ou produto para ideation"
PARAM platform AS STRING LIKE "instagram" DESCRIPTION "Plataforma alvo: instagram, facebook, whatsapp, email" OPTIONAL
PARAM count AS INTEGER LIKE 5 DESCRIPTION "Quantidade de ideias a gerar" OPTIONAL

DESCRIPTION "Gera ideias de conteúdo de marketing usando IA para campanhas."

IF NOT topic THEN
    TALK "Qual é o tema ou produto para gerar ideias?"
    HEAR topic AS STRING
END IF

platforms = IIF(platform, platform, "várias plataformas (Instagram, WhatsApp, Email)")
num_ideas = IIF(count, count, 5)

TALK "🤖 Gerando " + num_ideas + " ideias de conteúdo para: **" + topic + "**"
TALK "Plataforma: " + platforms
TALK ""

prompt = "Gere " + num_ideas + " ideias de conteúdo de marketing criativas e engagement para o tema: " + topic + ". "
prompt = prompt + "Para cada ideia, forneça: headline, mensagem-chave, call-to-action e hashtags relevantes. "
prompt = prompt + "Responda em formato de lista numerada."

ideas = LLM prompt

TALK "💡 **Ideias Geradas:**"
TALK ideas

TALK "Quer que eu salve algumas dessas ideias para usar depois?"
HEAR save_ideas AS BOOLEAN

IF save_ideas THEN
    TALK "Quais números das ideias quer salvar? (ex: 1,3,5)"
    HEAR selected AS STRING
    
    saved_count = 0
    FOR EACH num IN SPLIT(selected, ",")
        idea_entry = POST "/api/marketing/ideas", #{
            topic: topic,
            platform: platform,
            idea_number: VAL(TRIM(num)),
            saved_at: NOW()
        }
        saved_count = saved_count + 1
    NEXT
    
    TALK "✅ " + saved_count + " ideia(s) salva(s)!"
END IF
