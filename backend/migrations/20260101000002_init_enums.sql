-- Entry kinds. Additive changes only (ALTER TYPE ... ADD VALUE).

CREATE TYPE entry_kind AS ENUM ('snippet', 'command', 'note', 'link', 'doc');
