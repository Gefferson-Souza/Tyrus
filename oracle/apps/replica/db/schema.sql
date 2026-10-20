CREATE TABLE category (
  id SERIAL PRIMARY KEY,
  name VARCHAR(80) NOT NULL,
  slug VARCHAR(80) NOT NULL UNIQUE,
  created_at TIMESTAMPTZ(3) NOT NULL DEFAULT now()
);

CREATE TABLE product (
  id SERIAL PRIMARY KEY,
  sku VARCHAR(32) NOT NULL,             -- deliberately NOT unique: the load test re-posts the same body
  name VARCHAR(120) NOT NULL,
  description TEXT NULL,
  price NUMERIC(10,2) NOT NULL,
  stock INTEGER NOT NULL,
  active BOOLEAN NOT NULL DEFAULT true,
  category_id INTEGER NOT NULL REFERENCES category(id),
  created_at TIMESTAMPTZ(3) NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ(3) NOT NULL DEFAULT now()
);

CREATE TABLE order_item (
  id SERIAL PRIMARY KEY,
  product_id INTEGER NOT NULL REFERENCES product(id),
  quantity INTEGER NOT NULL,
  unit_price NUMERIC(10,2) NOT NULL,
  sold_at TIMESTAMPTZ(3) NOT NULL
);

CREATE INDEX order_item_sold_at_idx ON order_item(sold_at);
