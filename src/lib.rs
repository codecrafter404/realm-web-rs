use std::fmt::Display;

pub use ::bson;
use bson::{Document, oid::ObjectId};
use builder_pattern::Builder;
use reqwest::{StatusCode, header::{HeaderMap, HeaderName, HeaderValue}};
use serde::{Serialize, Deserialize};

#[derive(Builder, Debug, Clone)]
/// Implements all the api calls, but doesn't hold information about the selected collection or database
pub struct Client {
    #[into]
    /// The application id, which is inserted into the query url
    pub application_id: String,
    #[into]
    /// authentication using the `apiKey` header
    pub api_token: String,

    #[default(ApiVersion::v1)]
    pub api_version: ApiVersion,
    #[into]
    #[default(None)]
    /// should be none, if deployed globally
    /// or <Region>.<Cloud>
    pub deployment_region: Option<String>,
}
#[derive(Debug, Clone)]
pub enum ApiVersion {
    #[allow(non_camel_case_types)]
    v1,
}
#[allow(unused)]
impl Client {
    /// gets base url https://data.mongodb-api.com/app/<App ID>/endpoint/data/<API Version>
    fn get_url(&self) -> String {
        format!(
            "https://{}data.mongodb-api.com/app/{}/endpoint/data/{}",
            match &self.deployment_region {
                Some(x) => format!("{}.", x),
                None => "".into()
            },
            self.application_id,
            match &self.api_version {
                ApiVersion::v1 => "v1",
            }
        )
    }
    /// gets the base headers
    fn get_auth_headers(&self) -> HeaderMap {
        let mut header_map = HeaderMap::new();
        header_map.append(HeaderName::from_static("apikey"), HeaderValue::from_str(&self.api_token).unwrap());
        header_map.append(HeaderName::from_static("content-type"), HeaderValue::from_static("application/json"));
        header_map.append(HeaderName::from_static("accept"), HeaderValue::from_static("application/json"));
        header_map
    }

    /// # Find a Single Document
    /// 
    /// ### filter
    /// A [MongoDB Query Filter](https://www.mongodb.com/docs/manual/tutorial/query-documents/). The findOne action returns the first document in the collection that matches this filter.
    /// If you do not specify a filter, the action matches all document in the collection.
    /// 
    /// ### projection
    /// A [MongoDB Query Projection](https://www.mongodb.com/docs/manual/tutorial/project-fields-from-query-results/).
    /// Depending on the projection, the returned document will either omit specific fields or include only specified fields or values
    pub async fn find_one(
        &self,
        collection: Collection,
        filter: Option<Document>,
        projection: Option<Document>,
        http_client: &reqwest::Client
    ) -> Result<FindResponse, Error> {
        let req = FindRequest {
            collection,
            filter,
            projection,
            sort: None,
            limit: None,
            skip: None
        };

        let res = http_client.post(format!("{}/action/findOne", self.get_url()))
            .headers(self.get_auth_headers())
            .body(serde_json::to_string(&req).map_err(|x| Error {status_code: None, error: format!("Format error: {:?}", x)})?)
            .send()
            .await.map_err(|x| Error {status_code: None, error: format!("Failed to send request: {:?}", x)})?;

        if !res.status().is_success(){
            return Err(Error { status_code: Some(res.status()), error: format!("; content: {}", res.text().await.unwrap_or_default()) })
        }

        res.json::<FindResponse>().await.map_err(|x| Error {status_code: None, error: format!("Failed to deserialize response: {:?}", x)})
    }
    /// # Find Multiple Documents
    /// ### filter
    /// A [MongoDB Query Filter](https://www.mongodb.com/docs/manual/tutorial/query-documents/). The find action returns documents in the collection that match this filter.
    /// If you do not specify a filter, the action matches all document in the collection. If the filter matches more documents than the specified limit, the action only returns a subset of them. You can use skip in subsequent queries to return later documents in the result set.
    /// 
    /// ### projection
    /// A [MongoDB Query Projection](https://www.mongodb.com/docs/manual/tutorial/project-fields-from-query-results/).
    /// Depending on the projection, the returned document will either omit specific fields or include only specified fields or values
    /// 
    /// ### sort
    /// A [MongoDB Sort Expression](https://www.mongodb.com/docs/manual/reference/operator/aggregation/sort/).
    /// Matched documents are returned in ascending or descending order of the fields specified in the expression.
    /// 
    /// ### limit
    /// The maximum number of matched documents to include in the returned result set. Each request may return up to 50,000 documents.
    ///
    /// ### skip
    /// The number of matched documents to skip before adding matched documents to the result set.
    pub async fn find(
        &self,
        collection: Collection,
        filter: Option<Document>,
        projection: Option<Document>,
        sort: Option<Document>,
        limit: Option<i32>,
        skip: Option<i32>,
        http_client: &reqwest::Client
    ) -> Result<FindResponse, Error> {
        let req = FindRequest {
            collection,
            filter,
            projection,
            sort,
            limit,
            skip
        };

        let res = http_client.post(format!("{}/action/find", self.get_url()))
            .headers(self.get_auth_headers())
            .body(serde_json::to_string(&req).map_err(|x| Error {status_code: None, error: format!("Format error: {:?}", x)})?)
            .send()
            .await.map_err(|x| Error {status_code: None, error: format!("Failed to send request: {:?}", x)})?;

        if !res.status().is_success(){
            return Err(Error { status_code: Some(res.status()), error: format!("; content: {}", res.text().await.unwrap_or_default()) })
        }

        res.json::<FindResponse>().await.map_err(|x| Error {status_code: None, error: format!("Failed to deserialize response: {:?}", x)})
    }
    /// # Insert a Single Document
    /// 
    /// ### document
    /// An [EJSON](https://www.mongodb.com/docs/manual/reference/mongodb-extended-json/) document to insert into the collection.
    pub async fn insert_one(
        &self,
        collection: Collection,
        document: Document,
        http_client: &reqwest::Client
    ) -> Result<InsertResponse, Error> {
        let req = InsertRequest {
            collection,
            document: Some(document),
            documents: None
        };
        let res = http_client.post(format!("{}/action/insertOne", self.get_url()))
            .headers(self.get_auth_headers())
            .body(serde_json::to_string(&req).map_err(|x| Error {status_code: None, error: format!("Format error: {:?}", x)})?)
            .send()
            .await.map_err(|x| Error {status_code: None, error: format!("Failed to send request: {:?}", x)})?;

        if !res.status().is_success(){
            return Err(Error { status_code: Some(res.status()), error: format!("; content: {}", res.text().await.unwrap_or_default()) })
        }

        res.json::<InsertResponse>().await.map_err(|x| Error {status_code: None, error: format!("Failed to deserialize response: {:?}", x)})
    } 
    /// # Insert Multiple Documents
    /// 
    /// ### documents
    /// An array of one or more [EJSON](https://www.mongodb.com/docs/manual/reference/mongodb-extended-json/) documents to insert into the collection.
    pub async fn insert(
        &self,
        collection: Collection,
        documents: Vec<Document>,
        http_client: &reqwest::Client
    ) -> Result<InsertResponse, Error> {
        let req = InsertRequest {
            collection,
            document: None,
            documents: Some(documents)
        };
        let res = http_client.post(format!("{}/action/insertMany", self.get_url()))
            .headers(self.get_auth_headers())
            .body(serde_json::to_string(&req).map_err(|x| Error {status_code: None, error: format!("Format error: {:?}", x)})?)
            .send()
            .await.map_err(|x| Error {status_code: None, error: format!("Failed to send request: {:?}", x)})?;

        if !res.status().is_success(){
            return Err(Error { status_code: Some(res.status()), error: format!("; content: {}", res.text().await.unwrap_or_default()) })
        }

        res.json::<InsertResponse>().await.map_err(|x| Error {status_code: None, error: format!("Failed to deserialize response: {:?}", x)})
    }
    /// # Update a Single Document
    /// ### filter
    /// A [MongoDB Query Filter](https://www.mongodb.com/docs/manual/tutorial/query-documents/). The updateOne action modifies the first document in the collection that matches this filter.
    /// ### update
    /// A [MongoDB Update Expression](https://www.mongodb.com/docs/manual/tutorial/update-documents/) that specifies how to modify the matched document.
    /// ### upsert
    /// The upsert flag only applies if no documents match the specified filter. If true, the updateOne action inserts a new document that matches the filter with the specified update applied to it.
    pub async fn update_one(
        &self,
        collection: Collection,
        filter: Document,
        update: Document,
        upsert: Option<bool>,
        http_client: &reqwest::Client
    ) -> Result<UpdateResponse, Error> {
        let req = UpdateRequest {
            collection,
            filter,
            update,
            upsert
        };
        let res = http_client.post(format!("{}/action/updateOne", self.get_url()))
            .headers(self.get_auth_headers())
            .body(serde_json::to_string(&req).map_err(|x| Error {status_code: None, error: format!("Format error: {:?}", x)})?)
            .send()
            .await.map_err(|x| Error {status_code: None, error: format!("Failed to send request: {:?}", x)})?;

        if !res.status().is_success(){
            return Err(Error { status_code: Some(res.status()), error: format!("; content: {}", res.text().await.unwrap_or_default()) })
        }

        res.json::<UpdateResponse>().await.map_err(|x| Error {status_code: None, error: format!("Failed to deserialize response: {:?}", x)})
    }
    /// # Update Multiple Documents
    /// 
    /// ### filter
    /// A [MongoDB Query Filter](https://www.mongodb.com/docs/manual/tutorial/query-documents/). The updateMany action modifies the first document in the collection that matches this filter.
    /// ### update
    /// A [MongoDB Update Expression](https://www.mongodb.com/docs/manual/tutorial/update-documents/) that specifies how to modify the matched document.
    /// ### upsert
    /// The upsert flag only applies if no documents match the specified filter. If true, the updateMany action inserts a new document that matches the filter with the specified update applied to it.
    pub async fn update(
        &self,
        collection: Collection,
        filter: Document,
        update: Document,
        upsert: Option<bool>,
        http_client: &reqwest::Client
    ) -> Result<UpdateResponse, Error> {
        let req = UpdateRequest {
            collection,
            filter,
            update,
            upsert
        };
        let res = http_client.post(format!("{}/action/updateMany", self.get_url()))
            .headers(self.get_auth_headers())
            .body(serde_json::to_string(&req).map_err(|x| Error {status_code: None, error: format!("Format error: {:?}", x)})?)
            .send()
            .await.map_err(|x| Error {status_code: None, error: format!("Failed to send request: {:?}", x)})?;

        if !res.status().is_success(){
            return Err(Error { status_code: Some(res.status()), error: format!("; content: {}", res.text().await.unwrap_or_default()) })
        }

        res.json::<UpdateResponse>().await.map_err(|x| Error {status_code: None, error: format!("Failed to deserialize response: {:?}", x)})
    }

    /// # Replace a Single Document
    /// ### filter
    /// A [MongoDB Query Filter](https://www.mongodb.com/docs/manual/tutorial/query-documents/). The replaceOne action overwrites the first document in the collection that matches this filter.
    /// ### replacement
    /// An [EJSON](https://www.mongodb.com/docs/manual/reference/mongodb-extended-json/) document that overwrites the matched document.
    /// ### upsert
    /// The upsert flag only applies if no documents match the specified filter. If true, the replaceOne action inserts the replacement document.
    pub async fn replace_one(
        &self,
        collection: Collection,
        filter: Document,
        replacement: Document,
        upsert: Option<bool>,
        http_client: &reqwest::Client
    ) -> Result<ReplaceResponse, Error> {
        let req = ReplaceRequest {
            collection,
            filter,
            replacement,
            upsert
        };
        let res = http_client.post(format!("{}/action/replaceOne", self.get_url()))
            .headers(self.get_auth_headers())
            .body(serde_json::to_string(&req).map_err(|x| Error {status_code: None, error: format!("Format error: {:?}", x)})?)
            .send()
            .await.map_err(|x| Error {status_code: None, error: format!("Failed to send request: {:?}", x)})?;

        if !res.status().is_success(){
            return Err(Error { status_code: Some(res.status()), error: format!("; content: {}", res.text().await.unwrap_or_default()) })
        }

        res.json::<ReplaceResponse>().await.map_err(|x| Error {status_code: None, error: format!("Failed to deserialize response: {:?}", x)})
    }
    /// # Delete a Single Document
    /// 
    /// ### filter
    /// A [MongoDB Query Filter](https://www.mongodb.com/docs/manual/tutorial/query-documents/). The deleteOne action deletes the first document in the collection that matches this filter.
    pub async fn delete_one(
        &self,
        collection: Collection,
        filter: Document,
        http_client: &reqwest::Client
    ) -> Result<DeleteResponse, Error> {
        let req = DeleteRequest {
            collection,
            filter,
        };
        let res = http_client.post(format!("{}/action/deleteOne", self.get_url()))
            .headers(self.get_auth_headers())
            .body(serde_json::to_string(&req).map_err(|x| Error {status_code: None, error: format!("Format error: {:?}", x)})?)
            .send()
            .await.map_err(|x| Error {status_code: None, error: format!("Failed to send request: {:?}", x)})?;

        if !res.status().is_success(){
            return Err(Error { status_code: Some(res.status()), error: format!("; content: {}", res.text().await.unwrap_or_default()) })
        }

        res.json::<DeleteResponse>().await.map_err(|x| Error {status_code: None, error: format!("Failed to deserialize response: {:?}", x)})
    }
    /// # Delete Multiple Documents
    /// 
    /// ### filter
    /// A [MongoDB Query Filter](https://www.mongodb.com/docs/manual/tutorial/query-documents/). The deleteMany action deletes all documents in the collection that match this filter.
    pub async fn delete(
        &self,
        collection: Collection,
        filter: Document,
        http_client: &reqwest::Client
    ) -> Result<DeleteResponse, Error> {
        let req = DeleteRequest {
            collection,
            filter,
        };
        let res = http_client.post(format!("{}/action/deleteMany", self.get_url()))
            .headers(self.get_auth_headers())
            .body(serde_json::to_string(&req).map_err(|x| Error {status_code: None, error: format!("Format error: {:?}", x)})?)
            .send()
            .await.map_err(|x| Error {status_code: None, error: format!("Failed to send request: {:?}", x)})?;

        if !res.status().is_success(){
            return Err(Error { status_code: Some(res.status()), error: format!("; content: {}", res.text().await.unwrap_or_default()) })
        }

        res.json::<DeleteResponse>().await.map_err(|x| Error {status_code: None, error: format!("Failed to deserialize response: {:?}", x)})
    }
    /// # Run an Aggregation Pipeline
    /// 
    /// ### pipeline
    /// A [MongoDB Aggregation Pipeline](https://www.mongodb.com/docs/manual/core/aggregation-pipeline/).
    pub async fn aggregate(
        &self,
        collection: Collection,
        pipeline: Vec<Document>,
        http_client: &reqwest::Client
    ) -> Result<AggregationResponse, Error> {
        let req = AggregationRequest {
            collection,
            pipeline,
        };
        let res = http_client.post(format!("{}/action/aggregate", self.get_url()))
            .headers(self.get_auth_headers())
            .body(serde_json::to_string(&req).map_err(|x| Error {status_code: None, error: format!("Format error: {:?}", x)})?)
            .send()
            .await.map_err(|x| Error {status_code: None, error: format!("Failed to send request: {:?}", x)})?;

        if !res.status().is_success(){
            return Err(Error { status_code: Some(res.status()), error: format!("; content: {}", res.text().await.unwrap_or_default()) })
        }

        res.json::<AggregationResponse>().await.map_err(|x| Error {status_code: None, error: format!("Failed to deserialize response: {:?}", x)})
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct FindResponse {
    pub document: Option<Document>,
    pub documents: Option<Vec<Document>>
}
#[allow(unused)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FindRequest {
    #[serde(flatten)]
    collection: Collection,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<Document>,
    #[serde(skip_serializing_if = "Option::is_none")]
    projection: Option<Document>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<Document>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skip: Option<i32>
}

#[allow(unused)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct InsertRequest {
    #[serde(flatten)]
    collection: Collection,
    #[serde(skip_serializing_if = "Option::is_none")]
    document: Option<Document>,
    #[serde(skip_serializing_if = "Option::is_none")]
    documents: Option<Vec<Document>>
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct InsertResponse {
    pub inserted_id: Option<ObjectId>,
    pub inserted_ids: Option<Vec<ObjectId>>
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResponse {
    pub matched_count: i32,
    pub modified_count: i32,
    pub upserted_id: Option<ObjectId>
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceResponse {
    pub matched_count: i32,
    pub modified_count: i32,
    pub upserted_id: Option<ObjectId>
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResponse {
    pub deleted_count: i32,
}

#[allow(unused)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateRequest {
    #[serde(flatten)]
    collection: Collection,
    filter: Document,
    update: Document,
    #[serde(skip_serializing_if = "Option::is_none")]
    upsert: Option<bool>,
}

#[allow(unused)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReplaceRequest {
    #[serde(flatten)]
    collection: Collection,
    filter: Document,
    replacement: Document,
    #[serde(skip_serializing_if = "Option::is_none")]
    upsert: Option<bool>,
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregationResponse {
    pub documents: Vec<Document>
}

#[allow(unused)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteRequest {
    #[serde(flatten)]
    collection: Collection,
    filter: Document,
}

#[allow(unused)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AggregationRequest {
    #[serde(flatten)]
    collection: Collection,
    pipeline: Vec<Document>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Error {
    /// A statuscode, only available if the request gets denied
    status_code: Option<StatusCode>,
    error: String,
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StatusCode: {:?}; {}", self.status_code, self.error)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
/// holds information which collection to select
pub struct Collection {
    /// Atlas data source
    pub data_source: String,
    /// database name
    pub database: String,
    /// collection name
    pub collection: String,
}
