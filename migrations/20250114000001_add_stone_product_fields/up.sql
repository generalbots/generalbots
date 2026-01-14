-- Stone Pagamentos compatible fields for products
-- Adiciona campos para integração com Stone e outros gateways de pagamento brasileiros

-- Campos de identificação fiscal e tributária
ALTER TABLE products ADD COLUMN IF NOT EXISTS ncm VARCHAR(10);  -- Nomenclatura Comum do Mercosul
ALTER TABLE products ADD COLUMN IF NOT EXISTS cest VARCHAR(9);  -- Código Especificador da Substituição Tributária
ALTER TABLE products ADD COLUMN IF NOT EXISTS cfop VARCHAR(4);  -- Código Fiscal de Operações e Prestações
ALTER TABLE products ADD COLUMN IF NOT EXISTS origem INTEGER DEFAULT 0;  -- Origem da mercadoria (0=Nacional, 1-8=Importado)
ALTER TABLE products ADD COLUMN IF NOT EXISTS gtin VARCHAR(14);  -- Global Trade Item Number (EAN/UPC)
ALTER TABLE products ADD COLUMN IF NOT EXISTS gtin_tributavel VARCHAR(14);  -- GTIN da unidade tributável

-- Dimensões detalhadas (para cálculo de frete)
ALTER TABLE products ADD COLUMN IF NOT EXISTS peso_liquido DECIMAL(10,3);  -- Peso líquido em kg
ALTER TABLE products ADD COLUMN IF NOT EXISTS peso_bruto DECIMAL(10,3);  -- Peso bruto em kg
ALTER TABLE products ADD COLUMN IF NOT EXISTS largura DECIMAL(10,2);  -- Largura em cm
ALTER TABLE products ADD COLUMN IF NOT EXISTS altura DECIMAL(10,2);  -- Altura em cm
ALTER TABLE products ADD COLUMN IF NOT EXISTS comprimento DECIMAL(10,2);  -- Comprimento/Profundidade em cm
ALTER TABLE products ADD COLUMN IF NOT EXISTS volumes INTEGER DEFAULT 1;  -- Quantidade de volumes

-- Informações tributárias
ALTER TABLE products ADD COLUMN IF NOT EXISTS icms_cst VARCHAR(3);  -- Código de Situação Tributária ICMS
ALTER TABLE products ADD COLUMN IF NOT EXISTS icms_aliquota DECIMAL(5,2);  -- Alíquota ICMS
ALTER TABLE products ADD COLUMN IF NOT EXISTS ipi_cst VARCHAR(2);  -- CST do IPI
ALTER TABLE products ADD COLUMN IF NOT EXISTS ipi_aliquota DECIMAL(5,2);  -- Alíquota IPI
ALTER TABLE products ADD COLUMN IF NOT EXISTS pis_cst VARCHAR(2);  -- CST do PIS
ALTER TABLE products ADD COLUMN IF NOT EXISTS pis_aliquota DECIMAL(5,2);  -- Alíquota PIS
ALTER TABLE products ADD COLUMN IF NOT EXISTS cofins_cst VARCHAR(2);  -- CST do COFINS
ALTER TABLE products ADD COLUMN IF NOT EXISTS cofins_aliquota DECIMAL(5,2);  -- Alíquota COFINS

-- Campos para marketplace e e-commerce
ALTER TABLE products ADD COLUMN IF NOT EXISTS marca VARCHAR(100);  -- Marca/Fabricante
ALTER TABLE products ADD COLUMN IF NOT EXISTS modelo VARCHAR(100);  -- Modelo
ALTER TABLE products ADD COLUMN IF NOT EXISTS cor VARCHAR(50);  -- Cor principal
ALTER TABLE products ADD COLUMN IF NOT EXISTS tamanho VARCHAR(20);  -- Tamanho (P, M, G, 38, 40, etc)
ALTER TABLE products ADD COLUMN IF NOT EXISTS material VARCHAR(100);  -- Material principal
ALTER TABLE products ADD COLUMN IF NOT EXISTS genero VARCHAR(20);  -- Masculino, Feminino, Unissex

-- Controle de estoque avançado
ALTER TABLE products ADD COLUMN IF NOT EXISTS localizacao_estoque VARCHAR(100);  -- Localização no armazém
ALTER TABLE products ADD COLUMN IF NOT EXISTS lote VARCHAR(50);  -- Número do lote
ALTER TABLE products ADD COLUMN IF NOT EXISTS data_validade DATE;  -- Data de validade
ALTER TABLE products ADD COLUMN IF NOT EXISTS data_fabricacao DATE;  -- Data de fabricação
ALTER TABLE products ADD COLUMN IF NOT EXISTS estoque_minimo INTEGER DEFAULT 0;  -- Estoque mínimo para alerta
ALTER TABLE products ADD COLUMN IF NOT EXISTS estoque_maximo INTEGER;  -- Estoque máximo
ALTER TABLE products ADD COLUMN IF NOT EXISTS ponto_reposicao INTEGER;  -- Ponto de reposição

-- Preços e custos detalhados
ALTER TABLE products ADD COLUMN IF NOT EXISTS preco_promocional DECIMAL(15,2);  -- Preço promocional
ALTER TABLE products ADD COLUMN IF NOT EXISTS promocao_inicio TIMESTAMPTZ;  -- Início da promoção
ALTER TABLE products ADD COLUMN IF NOT EXISTS promocao_fim TIMESTAMPTZ;  -- Fim da promoção
ALTER TABLE products ADD COLUMN IF NOT EXISTS custo_frete DECIMAL(15,2);  -- Custo médio de frete
ALTER TABLE products ADD COLUMN IF NOT EXISTS margem_lucro DECIMAL(5,2);  -- Margem de lucro %

-- Campos Stone específicos
ALTER TABLE products ADD COLUMN IF NOT EXISTS stone_item_id VARCHAR(100);  -- ID do item na Stone
ALTER TABLE products ADD COLUMN IF NOT EXISTS stone_category_id VARCHAR(100);  -- ID da categoria na Stone
ALTER TABLE products ADD COLUMN IF NOT EXISTS stone_metadata JSONB DEFAULT '{}';  -- Metadados Stone

-- SEO e busca
ALTER TABLE products ADD COLUMN IF NOT EXISTS slug VARCHAR(255);  -- URL amigável
ALTER TABLE products ADD COLUMN IF NOT EXISTS meta_title VARCHAR(255);  -- Título para SEO
ALTER TABLE products ADD COLUMN IF NOT EXISTS meta_description TEXT;  -- Descrição para SEO
ALTER TABLE products ADD COLUMN IF NOT EXISTS tags TEXT[];  -- Tags para busca

-- Índices para novos campos
CREATE INDEX IF NOT EXISTS idx_products_ncm ON products(ncm);
CREATE INDEX IF NOT EXISTS idx_products_gtin ON products(gtin);
CREATE INDEX IF NOT EXISTS idx_products_marca ON products(marca);
CREATE INDEX IF NOT EXISTS idx_products_slug ON products(slug);
CREATE INDEX IF NOT EXISTS idx_products_validade ON products(data_validade);
CREATE INDEX IF NOT EXISTS idx_products_stone_item ON products(stone_item_id);

-- Também adicionar campos similares nas variantes
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS gtin VARCHAR(14);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS peso_liquido DECIMAL(10,3);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS peso_bruto DECIMAL(10,3);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS largura DECIMAL(10,2);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS altura DECIMAL(10,2);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS comprimento DECIMAL(10,2);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS cor VARCHAR(50);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS tamanho VARCHAR(20);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS images JSONB DEFAULT '[]';

CREATE INDEX IF NOT EXISTS idx_product_variants_gtin ON product_variants(gtin);
