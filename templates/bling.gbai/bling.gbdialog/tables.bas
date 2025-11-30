TABLE Contatos ON maria
    Id number key
    Nome string(150)
    Codigo string(50)
    Situacao string(5)
    NumeroDocumento string(25)
    Telefone string(20)
    Celular string(20)
    Fantasia string(150)
    Tipo string(5)
    IndicadorIe string(5)
    Ie string(22)
    Rg string(22)
    OrgaoEmissor string(22)
    Email string(50)
    Endereco_geral_endereco string(100)
    Endereco_geral_cep string(10)
    Endereco_geral_bairro string(50)
    Endereco_geral_municipio string(50)
    Endereco_geral_uf string(5)
    Endereco_geral_numero string(15)
    Endereco_geral_complemento string(50)
    Cobranca_endereco string(100)
    Cobranca_cep string(10)
    Cobranca_bairro string(50)
    Cobranca_municipio string(50)
    Cobranca_uf string(5)
    Cobranca_numero string(15)
    Cobranca_complemento string(50)
    Vendedor_id number
    DadosAdicionais_dataNascimento date
    DadosAdicionais_sexo string(5)
    DadosAdicionais_naturalidade string(25)
    Financeiro_limiteCredito double
    Financeiro_condicaoPagamento string(20)
    Financeiro_categoria_id number
    Pais_nome string(100)
END TABLE

TABLE Pedidos ON maria
    Id number key
    Numero integer
    NumeroLoja string(15)
    Data date
    DataSaida date
    DataPrevista date
    TotalProdutos double
    Desconto_valor double
    Desconto_unidade string(15)
    Contato_id number
    Total double
    Contato_nome string(150)
    Contato_tipoPessoa string(1)
    Contato_numeroDocumento string(20)
    Situacao_id integer
    Situacao_valor double
    Loja_id integer
    Vendedor_id number
    NotaFiscal_id number
END TABLE

TABLE PedidosItem ON maria
    Id number key
    Numero integer
    Sku string(20)
    Unidade string(8)
    Quantidade integer
    Desconto double
    Valor double
    Custo double
    AliquotaIPI double
    Descricao string(255)
    DescricaoDetalhada string(250)
    Produto_id number
    Pedido_id number
END TABLE

TABLE ProdutoImagem ON maria
    Id number key
    Ordinal number
    Sku string(20)
    Link string(250)
END TABLE

TABLE Produtos ON maria
    Id number key
    Nome string(150)
    Sku string(20)
    SkuPai string(20)
    Preco double
    Tipo string(1)
    Situacao string(1)
    Formato string(1)
    Hierarquia string(1)
    DescricaoCurta string(4000)
    DataValidade date
    Unidade string(5)
    PesoLiquido double
    PesoBruto double
    Volumes integer
    ItensPorCaixa integer
    Gtin string(50)
    GtinEmbalagem string(50)
    TipoProducao string(5)
    Condicao integer
    FreteGratis boolean
    Marca string(100)
    DescricaoComplementar string(4000)
    LinkExterno string(255)
    Observacoes string(255)
    Categoria_id integer
    Estoque_minimo integer
    Estoque_maximo integer
    Estoque_crossdocking integer
    Estoque_localizacao string(50)
    ActionEstoque string(50)
    Dimensoes_largura double
    Dimensoes_altura double
    Dimensoes_profundidade double
    Dimensoes_unidadeMedida double
    Tributacao_origem integer
    Tributacao_nFCI string(50)
    Tributacao_ncm string(50)
    Tributacao_cest string(50)
    Tributacao_codigoListaServicos string(50)
    Tributacao_spedTipoItem string(50)
    Tributacao_codigoItem string(50)
    Tributacao_percentualTributos double
    Tributacao_valorBaseStRetencao double
    Tributacao_valorStRetencao double
    Tributacao_valorICMSSubstituto double
    Tributacao_codigoExcecaoTipi string(50)
    Tributacao_classeEnquadramentoIpi string(50)
    Tributacao_valorIpiFixo double
    Tributacao_codigoSeloIpi string(50)
    Tributacao_valorPisFixo double
    Tributacao_valorCofinsFixo double
    Tributacao_dadosAdicionais string(50)
    GrupoProduto_id number
    Midia_video_url string(255)
    Midia_imagens_externas_0_link string(255)
    LinhaProduto_id number
    Estrutura_tipoEstoque string(5)
    Estrutura_lancamentoEstoque string(5)
    Estrutura_componentes_0_produto_id number
    Estrutura_componentes_0_produto_Quantidade double
END TABLE

TABLE Depositos ON maria
    Internal_Id number key
    Id number
    Sku string(20)
    SaldoFisico double
    SaldoVirtual double
END TABLE

TABLE Vendedores ON maria
    Id number key
    DescontoLimite double
    Loja_Id number
    Contato_Id number
    Contato_Nome string(100)
    Contato_Situacao string(1)
END TABLE

TABLE ProdutoFornecedor ON maria
    Id number key
    Descricao string(255)
    Codigo string(50)
    PrecoCusto double
    PrecoCompra double
    Padrao boolean
    Produto_id number
    Fornecedor_id number
    Garantia integer
END TABLE

TABLE ContasAPagar ON maria
    Id number key
    Situacao integer
    Vencimento date
    Valor double
    Contato_id number
    FormaPagamento_id number
    Saldo double
    DataEmissao date
    VencimentoOriginal date
    NumeroDocumento string(50)
    Competencia date
    Historico string(255)
    NumeroBanco string(10)
    Portador_id number
    Categoria_id number
    Borderos string(255)
    Ocorrencia_tipo integer
END TABLE

TABLE ContasAReceber ON maria
    Id number key
    Situacao integer
    Vencimento date
    Valor double
    IdTransacao string(50)
    LinkQRCodePix string(255)
    LinkBoleto string(255)
    DataEmissao date
    Contato_id number
    Contato_nome string(150)
    Contato_numeroDocumento string(20)
    Contato_tipo string(1)
    FormaPagamento_id number
    FormaPagamento_codigoFiscal integer
    ContaContabil_id number
    ContaContabil_descricao string(255)
    Origem_id number
    Origem_tipoOrigem string(20)
    Origem_numero string(20)
    Origem_dataEmissao date
    Origem_valor double
    Origem_situacao integer
    Origem_url string(255)
    Saldo double
    VencimentoOriginal date
    NumeroDocumento string(50)
    Competencia date
    Historico string(255)
    NumeroBanco string(10)
    Portador_id number
    Categoria_id number
    Vendedor_id number
    Borderos string(255)
    Ocorrencia_tipo integer
END TABLE

TABLE CategoriaReceita ON maria
    Id number key
    IdCategoriaPai number
    Descricao string(255)
    Tipo integer
    Situacao integer
END TABLE

TABLE FormaDePagamento ON maria
    Id number key
    Descricao string(255)
    TipoPagamento integer
    Situacao integer
    Fixa boolean
    Padrao integer
    Finalidade integer
    Condicao string(10)
    Destino integer
    Taxas_aliquota double
    Taxas_valor double
    Taxas_prazo integer
    DadosCartao_bandeira integer
    DadosCartao_tipo integer
    DadosCartao_cnpjCredenciadora string(16)
END TABLE

TABLE NaturezaDeOperacao ON maria
    Id number key
    Situacao integer
    Padrao integer
    Descricao string(255)
END TABLE

TABLE Parcela ON maria
    Id number key
    Pedido_id number
    DataVencimento date
    Valor double
    Observacoes string(255)
    FormaPagamento_id number
END TABLE

TABLE HistoricoPreco ON maria
    Id number key
    Sku string(50)
    PrecoAntigo double
    PrecoAtual double
    Produto_id number
    DataModificado date
END TABLE
