Of course. Granting the "Service Account Token Creator" role is a critical step for securely using short-lived credentials. It allows your user account (the one you're logged into IDX with) to generate temporary credentials for a designated service account, without needing to handle static key files.

Based on your project's documentation (docs/GCP_SHORT_LIVED_CREDENTIALS.md), you can do this with a single gcloud command.

You need to tell Google Cloud that your user account is allowed to create tokens for the target service account.

Run the following command in your terminal. Be sure to replace the placeholder values:

gcloud iam service-accounts add-iam-policy-binding \
  TARGET_SERVICE_ACCOUNT_EMAIL \
  --member="user:YOUR_EMAIL_ADDRESS" \
  --role="roles/iam.serviceAccountTokenCreator"



TARGET_SERVICE_ACCOUNT_EMAIL: Replace this with the full email address of the service account that cloudlist will use for discovery.

Example: cloudlist-discovery-sa@my-gcp-project-123.iam.gserviceaccount.com
YOUR_EMAIL_ADDRESS: Replace this with the email address you use to log into Google Cloud and IDX.

Example: your.name@gmail.com
Let's say your email is dev@example.com and your service account is scanner@my-project.iam.gserviceaccount.com. The command would be:

gcloud iam service-accounts add-iam-policy-binding \
  scanner@my-project.iam.gserviceaccount.com \
  --member="user:dev@example.com" \
  --role="roles/iam.serviceAccountTokenCreator"



After running this, your user account has the necessary permission to impersonate that service account, and the use_short_lived_credentials: true flag in your provider-config.yaml will now work as expected.