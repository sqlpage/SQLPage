SELECT 'shell' as component,
'style_pricing.css' as css ;


SELECT 'hero' as component,
    'DATAPAGE PRICING PLANS' as title,
'
> *Start free, launch with fixed costs, and scale efficiently.*   

> If you have any questions regarding **DataPage.app**, fill out the form [*here*](https://beta.datapage.app/fill-the-form.sql) and we''ll get back to you shortly.' as description_md;

SELECT 'START PLAN' as title,
'
### **Price**: **€18/month** *(First 1 month FREE)*
### **🚩[Register for the *START Plan*](https://buy.stripe.com/9AQeWCa6k85Q9gY8wy)**
---
- **Database Size**: **128MB**
- **Ideal For**: Testing and small-scale projects.
- **Features**:
  - Basic SQLPage hosting.
  - Essential components for simple applications.
  - Community Support via forums.
---
### **🚩[Register for the *START Plan*](https://buy.stripe.com/9AQeWCa6k85Q9gY8wy)**
'
as description_md,
    'player-play' as icon,
    'blue' as color;

SELECT 'PRO PLAN' as title,
'
### **Price**: **€40/month** *(First 1 month FREE)*
### **🚩[Register for the *PRO Plan*](https://buy.stripe.com/eVabKqces99U1OweUX)**
---
- **Database Size**: **1GB**
- **Ideal For**: Growing projects and businesses needing enhanced support and features
- **Features**:
  - All *START plan* features.
  - **Priority support**: Get faster response times and direct assistance from our support team
  - **Custom Domain**: Use your custom domain name with your SQLPage app
---

### **🚩[Register for the *PRO Plan*](https://buy.stripe.com/eVabKqces99U1OweUX)**
'
 as description_md,
    'shield-check' as icon,
    'green' as color;



  
SELECT 'ENTREPRISE PLAN' as title,
'
### **Price**: **€600/month** *(First 1 month FREE)*
### **🚩[Register for the *ENTREPRISE Plan*](https://buy.stripe.com/8wM6q62DS5XI3WE4gk)**
---
- **Database**: **Custom Scaling**
- **Ideal For**: Large-scale operations with custom needs.
- **Features**:
  - All Pro Plan features.
  - **Custom Deployment**: Tailored deployment to suit your specific requirements, whether on-premises or in the cloud.
  - **Database Scaling**: Dynamically scale your database to handle increased traffic and storage needs.
  - **Authentication**: Implement OpenID Connect and OAuth2 for secure user authentication via Google, Facebook, or internal company accounts.
  - **Premium Components**: Access to exclusive, high-performance components for building complex applications.
  - **1-Hour Monthly Support**: Dedicated one-on-one support session with our experts each month.
  - **SLA Agreement**: Service Level Agreement with guaranteed uptime and response times.
  - **Custom Integration**: Personalized integration with your existing systems and workflows.
  - **Onboarding Assistance**: Get personalized setup and onboarding assistance for a smooth start.
---

### **🚩[Register for the *ENTREPRISE Plan*](https://buy.stripe.com/8wM6q62DS5XI3WE4gk)**
  ' as description_md,
    'bubble-plus' as icon,
    'red' as color;

SELECT 'text' as component,
'' as title,
'## **Ready to Get Started?**
[Sign Up Now](https://datapage.app) and start building your SQLPage app with Datapage.app today!' as contents_md;
