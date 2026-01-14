-- Rollback Stone Pagamentos fields from products table

-- Remove índices das variantes
DROP INDEX IF EXISTS idx_product_variants_gtin;

-- Remove campos das variantes
ALTER TABLE product_variants DROP COLUMN IF EXISTS gtin;
ALTER TABLE product_variants DROP COLUMN IF EXISTS peso_liquido;
ALTER TABLE product_variants DROP COLUMN IF EXISTS peso_bruto;
ALTER TABLE product_variants DROP COLUMN IF EXISTS largura;
ALTER TABLE product_variants DROP COLUMN IF EXISTS altura;
ALTER TABLE product_variants DROP COLUMN IF EXISTS comprimento;
ALTER TABLE product_variants DROP COLUMN IF EXISTS cor;
ALTER TABLE product_variants DROP COLUMN IF EXISTS tamanho;
ALTER TABLE product_variants DROP COLUMN IF EXISTS images;

-- Remove índices dos produtos
DROP INDEX IF EXISTS idx_products_ncm;
DROP INDEX IF EXISTS idx_products_gtin;
DROP INDEX IF EXISTS idx_products_marca;
DROP INDEX IF EXISTS idx_products_slug;
DROP INDEX IF EXISTS idx_products_validade;
DROP INDEX IF EXISTS idx_products_stone_item;

-- Remove campos SEO e busca
ALTER TABLE products DROP COLUMN IF EXISTS slug;
ALTER TABLE products DROP COLUMN IF EXISTS meta_title;
ALTER TABLE products DROP COLUMN IF EXISTS meta_description;
ALTER TABLE products DROP COLUMN IF EXISTS tags;

-- Remove campos Stone específicos
ALTER TABLE products DROP COLUMN IF EXISTS stone_item_id;
ALTER TABLE products DROP COLUMN IF EXISTS stone_category_id;
ALTER TABLE products DROP COLUMN IF EXISTS stone_metadata;

-- Remove preços e custos detalhados
ALTER TABLE products DROP COLUMN IF EXISTS preco_promocional;
ALTER TABLE products DROP COLUMN IF EXISTS promocao_inicio;
ALTER TABLE products DROP COLUMN IF EXISTS promocao_fim;
ALTER TABLE products DROP COLUMN IF EXISTS custo_frete;
ALTER TABLE products DROP COLUMN IF EXISTS margem_lucro;

-- Remove controle de estoque avançado
ALTER TABLE products DROP COLUMN IF EXISTS localizacao_estoque;
ALTER TABLE products DROP COLUMN IF EXISTS lote;
ALTER TABLE products DROP COLUMN IF EXISTS data_validade;
ALTER TABLE products DROP COLUMN IF EXISTS data_fabricacao;
ALTER TABLE products DROP COLUMN IF EXISTS estoque_minimo;
ALTER TABLE products DROP COLUMN IF EXISTS estoque_maximo;
ALTER TABLE products DROP COLUMN IF EXISTS ponto_reposicao;

-- Remove campos para marketplace e e-commerce
ALTER TABLE products DROP COLUMN IF EXISTS marca;
ALTER TABLE products DROP COLUMN IF EXISTS modelo;
ALTER TABLE products DROP COLUMN IF EXISTS cor;
ALTER TABLE products DROP COLUMN IF EXISTS tamanho;
ALTER TABLE products DROP COLUMN IF EXISTS material;
ALTER TABLE products DROP COLUMN IF EXISTS genero;

-- Remove informações tributárias
ALTER TABLE products DROP COLUMN IF EXISTS icms_cst;
ALTER TABLE products DROP COLUMN IF EXISTS icms_aliquota;
ALTER TABLE products DROP COLUMN IF EXISTS ipi_cst;
ALTER TABLE products DROP COLUMN IF EXISTS ipi_aliquota;
ALTER TABLE products DROP COLUMN IF EXISTS pis_cst;
ALTER TABLE products DROP COLUMN IF EXISTS pis_aliquota;
ALTER TABLE products DROP COLUMN IF EXISTS cofins_cst;
ALTER TABLE products DROP COLUMN IF EXISTS cofins_aliquota;

-- Remove dimensões detalhadas
ALTER TABLE products DROP COLUMN IF EXISTS peso_liquido;
ALTER TABLE products DROP COLUMN IF EXISTS peso_bruto;
ALTER TABLE products DROP COLUMN IF EXISTS largura;
ALTER TABLE products DROP COLUMN IF EXISTS altura;
ALTER TABLE products DROP COLUMN IF EXISTS comprimento;
ALTER TABLE products DROP COLUMN IF EXISTS volumes;

-- Remove campos de identificação fiscal e tributária
ALTER TABLE products DROP COLUMN IF EXISTS ncm;
ALTER TABLE products DROP COLUMN IF EXISTS cest;
ALTER TABLE products DROP COLUMN IF EXISTS cfop;
ALTER TABLE products DROP COLUMN IF EXISTS origem;
ALTER TABLE products DROP COLUMN IF EXISTS gtin;
ALTER TABLE products DROP COLUMN IF EXISTS gtin_tributavel;
