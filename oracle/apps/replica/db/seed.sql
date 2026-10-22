-- Deterministic seed: generate_series plus md5-derived pseudo-randomness. Never random().
-- seed_hash(k) is an unsigned 32-bit value derived from md5(k).
CREATE FUNCTION seed_hash(k text) RETURNS bigint
  LANGUAGE sql IMMUTABLE
  AS $$ SELECT ('x' || substr(md5(k), 1, 8))::bit(32)::bigint $$;

-- 20 categories (real Portuguese names, accents included) with fixed timestamps.
INSERT INTO category (id, name, slug, created_at)
SELECT c.id, c.name, c.slug,
       timestamptz '2025-01-01 08:00:00+00' + c.id * interval '1 day 02:03:04.567'
FROM (VALUES
  (1,  'Açougue',                   'acougue'),
  (2,  'Hortifrúti',                'hortifruti'),
  (3,  'Laticínios',                'laticinios'),
  (4,  'Padaria',                   'padaria'),
  (5,  'Bebidas',                   'bebidas'),
  (6,  'Mercearia',                 'mercearia'),
  (7,  'Limpeza',                   'limpeza'),
  (8,  'Higiene e Beleza',          'higiene-e-beleza'),
  (9,  'Congelados',                'congelados'),
  (10, 'Frios e Embutidos',         'frios-e-embutidos'),
  (11, 'Biscoitos e Salgadinhos',   'biscoitos-e-salgadinhos'),
  (12, 'Massas e Molhos',           'massas-e-molhos'),
  (13, 'Enlatados e Conservas',     'enlatados-e-conservas'),
  (14, 'Cereais e Matinais',        'cereais-e-matinais'),
  (15, 'Pet Shop',                  'pet-shop'),
  (16, 'Utilidades Domésticas',     'utilidades-domesticas'),
  (17, 'Papelaria',                 'papelaria'),
  (18, 'Doces e Chocolates',        'doces-e-chocolates'),
  (19, 'Adega e Vinhos',            'adega-e-vinhos'),
  (20, 'Pescados e Frutos do Mar',  'pescados-e-frutos-do-mar')
) AS c(id, name, slug);
SELECT setval('category_id_seq', 20, true);

-- 10,000 products.
INSERT INTO product (id, sku, name, description, price, stock, active, category_id, created_at, updated_at)
SELECT
  i,
  'PRD-' || lpad(i::text, 6, '0'),
  (ARRAY['Premium','Orgânico','Integral','Tradicional','Light','Zero','Especial','Artesanal',
         'Caseiro','Fresco','Natural','Defumado','Crocante','Cremoso','Extra','Super'])[1 + (seed_hash('adj' || i) % 16)::int]
    || ' ' ||
  (ARRAY['Pão Francês','Café Torrado','Feijão Carioca','Açúcar Refinado','Arroz Branco','Macarrão Espaguete',
         'Molho de Tomate','Leite Integral','Queijo Mussarela','Presunto Cozido','Linguiça Calabresa','Maçã Fuji',
         'Banana Prata','Mamão Papaya','Água Mineral','Refrigerante Guaraná','Suco de Laranja','Sabão em Pó',
         'Detergente Neutro','Shampoo Hidratante','Sorvete de Chocolate','Biscoito Maisena','Chocolate ao Leite',
         'Azeite Extra Virgem','Farinha de Trigo','Óleo de Soja','Pão de Queijo','Iogurte Natural',
         'Manteiga com Sal','Pimentão Verde'])[1 + (seed_hash('noun' || i) % 30)::int]
    || ' ' ||
  (ARRAY['200g','500g','1kg','1,5L','350ml','2L','900g','12un','5kg','750ml'])[1 + (seed_hash('size' || i) % 10)::int]
    || CASE WHEN seed_hash('emoji' || i) % 250 = 0
            THEN (ARRAY[' 🍎',' 🥖',' 🧀',' 🍫'])[1 + (seed_hash('emojipick' || i) % 4)::int]
            ELSE '' END,
  CASE WHEN seed_hash('desc-null' || i) % 10 < 3 THEN NULL
       ELSE
         (ARRAY['Produto de alta qualidade.',
                'Contém glúten.',
                'Validade: 6 meses após a abertura.',
                'Linha "Gourmet" — edição limitada.',
                E'Armazenar em local seco\ne arejado.',
                'Origem: São Paulo/SP. Preço promocional até o fim do mês!',
                'Sem conservantes; açúcar & sal reduzidos <light>.',
                E'Caminho C:\\estoque\\item (legado).',
                E'Coluna A\tColuna B',
                'Ótimo para crianças 🍼 e adultos.'])[1 + (seed_hash('d1' || i) % 10)::int]
         || CASE WHEN seed_hash('d2on' || i) % 2 = 0
                 THEN ' ' || (ARRAY['Conservar refrigerado.',
                                    'Não congelar.',
                                    'Ação: 3 por 2.',
                                    'Importado.',
                                    'Fabricação própria.'])[1 + (seed_hash('d2' || i) % 5)::int]
                 ELSE '' END
  END,
  round((0.5 + (seed_hash('price' || i) % 99950) / 100.0)::numeric, 2),
  (seed_hash('stock' || i) % 500)::int,
  seed_hash('active' || i) % 10 <> 0,
  1 + (seed_hash('cat' || i) % 20)::int,
  timestamptz '2025-03-01 00:00:00+00' + i * interval '00:02:07.321'
    + (seed_hash('cms' || i) % 1000) * interval '1 millisecond',
  timestamptz '2025-03-01 00:00:00+00' + i * interval '00:02:07.321'
    + (seed_hash('cms' || i) % 1000) * interval '1 millisecond'
    + (seed_hash('ums' || i) % 86400000) * interval '1 millisecond'
FROM generate_series(1, 10000) AS i
ORDER BY i;
SELECT setval('product_id_seq', 10000, true);

-- 50,000 order items over 2026 (365 days / 50,000 = 630,720 ms per slot, so about 137 per day).
INSERT INTO order_item (id, product_id, quantity, unit_price, sold_at)
SELECT
  s.i,
  s.product_id,
  1 + (seed_hash('qty' || s.i) % 12)::int,
  round(p.price * (90 + seed_hash('var' || s.i) % 21) / 100.0, 2),
  timestamptz '2026-01-01 00:00:00+00'
    + (s.i - 1)::bigint * 630720 * interval '1 millisecond'
    + (seed_hash('sold' || s.i) % 630720) * interval '1 millisecond'
FROM (
  SELECT i, 1 + (seed_hash('prod' || i) % 10000)::int AS product_id
  FROM generate_series(1, 50000) AS i
) AS s
JOIN product p ON p.id = s.product_id
ORDER BY s.i;
SELECT setval('order_item_id_seq', 50000, true);

DROP FUNCTION seed_hash(text);
VACUUM (ANALYZE) category;
VACUUM (ANALYZE) product;
VACUUM (ANALYZE) order_item;
