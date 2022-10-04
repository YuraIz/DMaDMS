# Yuri Izmer 053501
database design on the topic: Importer of food products

## Requirements

1. Compulsory functional requirements:

    - User Authorization
    - User Management (CRUD)
    - Role System (Admin, Manager, Client, Supplier)
    - Logging

1. The database should be capable of supporting the following maintenance
   transactions: 
	- Create and maintain records recording the details of countries, suppliers, clients. (Admin) 
	- Create and maintain product categories and subcategories. (Admin, Manager) 
	- Create new product categories. (Admin, Manager, Supplier) 
	- Create and maintain records recording the details of products. (Admin, Manager, Supplier) 
	- Create and maintain records recording
   the details of warehouses and product locations.
   (Admin, Manager) 
	- Create and maintain records recording
   the details of client addresses and product requirements.
   (Admin, Manager, Client)

1. The database should be capable of supporting the following example query transactions:
	- List suppliers with name, email and country.
	- List products for category or subcategory.
	- List product requierments for each client address.
	- List awailable products per warehouse.
	- List countries, categories and subcategories.
	- List users.
	- List all addresses for client.

## Entitity relationship diagram

![image](img/entities.svg)

## Entities

> Note: all fields are non-nullable except those marked as nullable

1. User

    - **user_id**: number, primary key
    - _supplier_id_: number, nullable and unique foreign key that refers to supplier
    - _client_id_: number, nullable and unique foreign key that refers to client
    - _user_role_id_: number, foreign key that refers to user role
    - name: varchar(320)
    - password: varchar(256)

1. User role - admin, manager, client, supplier, etc...

    - **user_role_id**: number, primary key
    - name: varchar(20)

1. Client - company that buys products

    - **client_id**: number, primary key
    - name: varchar(320)
    - email: varchar(320)

1. Client address - one of the client addresses

    - **client_address_id**: number, primary key
    - _client_id_: number, foreign key that refers to client
    - address: varchar

1. Product requirement - required count of product for address

    - **product_requirement_id**: number, primary key
    - _product_id_: number, foreign key that refers to product
    - _client_address_id_: number, foreign key that refers to client address
    - count: number

1. Supplier - company that sells products

    - **supplier_id**: number, primary key
    - _country_id_: number, foreign key that refers to country
    - name: varchar(320), name of supplier
    - email: varchar(320), email of supplier

1. Country

    - **country_id**: number, primary key
    - name: varchar(60), name of the country

1. Product

    - **product_id**: number, primary key
    - _supplier_id_: number, foreign key that refers to supplier
    - _subcategory_id_: number, foreign key that refers to subcategory
    - name: varchar, name of product

1. Product subcategory

    - **subcategory_id**: number, primary key
    - _category_id_: number, foreign key that refers to category
    - name: varchar, name of subcategory

1. Product category - grocery, canned food, healthy food etc...

    - **category_id**: number, primary key
    - name: varchar, name of the category

1. Warehouse - place where products can be located

    - **warehouse_id**: number, primary key
    - address: varchar, name of the category

1. Product locations
    - **product_location_id**: number, primary key
    - _warehouse_id_: number, foreign key that refers to warehouse
    - _product_id_: number, foreign key that refers to product
    - count: number
