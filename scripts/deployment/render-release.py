"""Render release operations. Credentials remain in environment/process memory."""
import argparse
import json
import os
import urllib.error
import urllib.parse
import urllib.request

NAMES = {"ndith-backend", "ndith-news", "ndith-frontend"}


def request(path, method="GET", body=None):
    key = os.environ.get("RENDER_API_KEY", "")
    if not key:
        raise SystemExit("RENDER_API_KEY is not configured")
    req = urllib.request.Request("https://api.render.com/v1/" + path, method=method,
        data=None if body is None else json.dumps(body).encode(),
        headers={"Authorization": "Bearer " + key, "Content-Type": "application/json", "Accept": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=45) as response:
            raw = response.read()
            return json.loads(raw) if raw else None
    except urllib.error.HTTPError as error:
        raise SystemExit(f"Render {method} {path.split('?')[0]} returned HTTP {error.code}; response body withheld") from None


def services():
    result = []
    cursor = None
    for _ in range(20):
        query = {"limit": 100}
        if cursor:
            query["cursor"] = cursor
        page = request("services?" + urllib.parse.urlencode(query))
        if not isinstance(page, list):
            raise SystemExit("Unexpected Render service-list format")
        result.extend(entry.get("service", entry) for entry in page)
        if len(page) < 100:
            break
        cursor = page[-1].get("cursor")
        if not cursor:
            raise SystemExit("Missing Render pagination cursor")
    return [service for service in result if service.get("name") in NAMES]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("operation", choices=["inspect", "deploy"])
    parser.add_argument("--service", choices=sorted(NAMES))
    parser.add_argument("--sha")
    args = parser.parse_args()
    found = services()
    if args.operation == "inspect":
        for service in found:
            details = service.get("serviceDetails", {})
            variables = request(f"services/{service['id']}/env-vars?limit=100")
            names = sorted(item.get("envVar", item).get("key", "") for item in variables)
            deployments = request(f"services/{service['id']}/deploys?limit=1")
            latest = deployments[0].get("deploy", deployments[0]) if deployments else {}
            print(json.dumps({"name": service["name"], "id": service["id"], "suspended": service.get("suspended"),
                "url": details.get("url"), "image": service.get("image"), "envNames": names,
                "latestDeploy": {key: latest.get(key) for key in ["id", "status", "image", "commit"]}}))
        return
    if not args.service or not args.sha or len(args.sha) != 40 or any(c not in "0123456789abcdef" for c in args.sha):
        raise SystemExit("Deploy requires a service and full lowercase commit SHA")
    matches = [service for service in found if service["name"] == args.service]
    if len(matches) != 1:
        raise SystemExit("Expected exactly one matching Render service")
    service = matches[0]
    prefix = "news-" if args.service == "ndith-news" else ""
    repo = "ndith-frontend" if args.service == "ndith-frontend" else "ndith-backend"
    image = f"ghcr.io/kcirtapfromspace/{repo}:{prefix}{args.sha[:7]}"
    result = request(f"services/{service['id']}/deploys", "POST", {"imageUrl": image})
    print(json.dumps({"service": args.service, "image": image, "deployId": result.get("id"), "status": result.get("status")}))


if __name__ == "__main__":
    main()
